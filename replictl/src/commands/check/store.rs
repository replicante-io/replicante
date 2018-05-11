use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use error_chain::ChainedError;
use prometheus::Registry;

use replicante::Config;
use replicante_data_store::Cursor;
use replicante_data_store::Error as StoreError;
use replicante_data_store::ErrorKind as StoreErrorKind;
use replicante_data_store::ValidationResult;
use replicante_data_store::Validator;

use super::super::super::Interfaces;
use super::super::super::Result;
use super::super::super::ResultExt;

use super::super::super::outcome::Error;
use super::super::super::outcome::Outcomes;
use super::super::super::outcome::Warning;


pub const COMMAND: &'static str = "store";

const COMMAND_DATA: &'static str = "data";
const COMMAND_SCHEMA: &'static str = "schema";
const FAILED_CHECK_SCHEMA : &'static str = "Failed to check store schema";
const FAILED_CHECK_DATA : &'static str = "Failed to check store data";

const MODEL_AGENT: &'static str = "Agent";
const MODEL_AGENT_INFO: &'static str = "AgentInfo";
const MODEL_CLUSTER_META: &'static str = "ClusterMeta";
const MODEL_CLUSTER_DISCOVERY: &'static str = "ClusterDiscovery";
const MODEL_EVENT: &'static str = "Event";
const MODEL_NODE: &'static str = "Node";
const MODEL_SHARD: &'static str = "Shard";


/// Configure the `replictl check store` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check the primary store for incompatibilities")
        .subcommand(SubCommand::with_name(COMMAND_DATA).about(
            "Check ALL primary store content for compatibility with this version of replicante"
        ))
        .subcommand(SubCommand::with_name(COMMAND_SCHEMA)
            .about("Check the primary store schema compatibility with this version of replicante")
        )
}


/// Check the primary store for incompatibilities.
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let command = args.subcommand_matches(super::COMMAND).unwrap();
    let command = command.subcommand_matches(COMMAND).unwrap();
    let command = command.subcommand_name().clone();
    match command {
        Some(COMMAND_DATA) => data(args, interfaces),
        Some(COMMAND_SCHEMA) => schema(args, interfaces),
        None => Err("Need a store check to run".into()),
        _ => Err("Received unrecognised command".into()),
    }
}


/// Check ALL primary store content for compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * Each content item is loaded and parsed.
pub fn data<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking store data");
    let confirm = interfaces.prompt().confirm_danger(
        "About to scan ALL content of the store. \
        This could impact your production system. \
        Would you like to proceed?"
    )?;
    if !confirm {
        error!(logger, "Cannot check without user interactive confirmation");
        return Err("Operation aborded by the user".into());
    }

    let mut outcomes = Outcomes::new();
    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).chain_err(|| FAILED_CHECK_SCHEMA)?;
    let registry = Registry::new();
    let store = Validator::new(config.storage, logger.clone(), &registry)
        .chain_err(|| FAILED_CHECK_DATA)?;

    info!(logger, "Checking records for the '{}' model", MODEL_AGENT);
    scan_collection(
        store.agents_count(), store.agents(),
        MODEL_AGENT, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    info!(logger, "Checking records for the '{}' model", MODEL_AGENT_INFO);
    scan_collection(
        store.agents_info_count(), store.agents_info(),
        MODEL_AGENT_INFO, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    info!(logger, "Checking records for the '{}' model", MODEL_CLUSTER_META);
    scan_collection(
        store.clusters_meta_count(), store.clusters_meta(),
        MODEL_CLUSTER_META, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    info!(logger, "Checking records for the '{}' model", MODEL_CLUSTER_DISCOVERY);
    scan_collection(
        store.cluster_discoveries_count(), store.cluster_discoveries(),
        MODEL_CLUSTER_DISCOVERY, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    info!(logger, "Checking records for the '{}' model", MODEL_EVENT);
    scan_collection(
        store.events_count(), store.events(),
        MODEL_EVENT, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    info!(logger, "Checking records for the '{}' model", MODEL_NODE);
    scan_collection(
        store.nodes_count(), store.nodes(),
        MODEL_NODE, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    info!(logger, "Checking records for the '{}' model", MODEL_SHARD);
    scan_collection(
        store.shards_count(), store.shards(),
        MODEL_SHARD, &mut outcomes, interfaces
    );
    outcomes.report(&logger);

    // Report results.
    if outcomes.has_errors() {
        error!(logger, "Store data checks failed");
        return Err("Store data checks failed".into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Store data checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Store data checks passed");
    Ok(())
}

fn scan_collection<Model: ::std::fmt::Debug>(
    count: ::replicante_data_store::Result<u64>,
    cursor: ::replicante_data_store::Result<Cursor<Model>>,
    collection: &str, outcomes: &mut Outcomes, interfaces: &Interfaces
) {
    let count = match count {
        Ok(count) => count,
        Err(error) => {
            let error = error.display_chain().to_string();
            outcomes.error(Error::GenericError(error));
            return;
        }
    };
    let cursor = match cursor {
        Ok(cursor) => cursor,
        Err(error) => {
            let error = error.display_chain().to_string();
            outcomes.error(Error::GenericError(error));
            return;
        }
    };
    let progress = interfaces.progress(Some(count));
    for item in progress.wrap_iter(cursor) {
        match item {
            Err(StoreError(StoreErrorKind::UnableToParseModel(id, msg), _)) => {
                outcomes.error(Error::UnableToParseModel(collection.to_string(), id, msg));
            },
            Err(error) => {
                let error = error.display_chain().to_string();
                outcomes.error(Error::GenericError(error));
            },
            Ok(_) => (),
        };
    }
    progress.finish();
}


/// Check the primary store schema compatibility with this version of replicante.
///
/// The following checks are performed:
///
///   * All needed collections/tables exist and have the correct schema.
///   * All needed and recommended indexes exist.
///   * No dropped collections/tables or indexes exist.
pub fn schema<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking store schema");

    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config).chain_err(|| FAILED_CHECK_SCHEMA)?;
    let registry = Registry::new();
    let store = Validator::new(config.storage, logger.clone(), &registry)
        .chain_err(|| FAILED_CHECK_SCHEMA)?;
    let mut outcomes = Outcomes::new();

    debug!(logger, "Checking schema");
    match store.schema() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.chain_err(|| "Failed to validate store schema")
                .display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
    };
    outcomes.report(&logger);

    debug!(logger, "Checking indexes");
    match store.indexes() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.chain_err(|| "Failed to validate store indexes")
                .display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
    };
    outcomes.report(&logger);

    debug!(logger, "Checking for removed collections/tables or indexes");
    match store.removed() {
        Ok(results) => consume_results(results, &mut outcomes),
        Err(error) => {
            let error = error.chain_err(|| "Failed to check for removed collections or indexes")
                .display_chain().to_string();
            outcomes.error(Error::GenericError(error));
        }
    };
    outcomes.report(&logger);

    // Finish up.
    if outcomes.has_errors() {
        error!(logger, "Store schema checks failed");
        return Err("Store schema checks failed".into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Store schema checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Store schema checks passed");
    Ok(())
}


fn consume_results(results: Vec<ValidationResult>, outcomes: &mut Outcomes) {
    for result in results {
        if result.error {
            outcomes.error(Error::StoreValidationError(result));
        } else {
            outcomes.warn(Warning::StoreValidationWarning(result));
        }
    }
}