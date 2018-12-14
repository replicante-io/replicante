use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use failure::ResultExt;
use failure::err_msg;
use slog::Logger;

use replicante::Config;
use replicante_coordinator::Admin;

use super::super::super::ErrorKind;
use super::super::super::Interfaces;
use super::super::super::Result;

use super::super::super::outcome::Error;
use super::super::super::outcome::Outcomes;


pub const COMMAND: &str = "coordinator";


/// Iterate over registered nodes to ensure they can be read.
fn check_registry(
    admin: &Admin, outcomes: &mut Outcomes, interfaces: &Interfaces, logger: &Logger
) -> Result<()> {
    info!(logger, "Checking registered nodes");
    let mut tracker = interfaces.progress("Processed more nodes");
    for node in admin.nodes() {
        if let Err(error) = node {
            let error = error.to_string();
            outcomes.error(Error::GenericError(error));
        }
        tracker.track();
    }
    outcomes.report(logger);
    Ok(())
}


/// Configure the `replictl check coordinator` command parser.
pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND)
        .about("Check all coordination data for incompatibilities")
}


/// Check all coordination data for incompatibilities
pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<()> {
    let logger = interfaces.logger();
    info!(logger, "Checking coordination data");
    let confirm = interfaces.prompt().confirm_danger(
        "About to scan ALL data in the distibuted coordination system. \
        This could impact your production system. \
        Would you like to proceed?"
    )?;
    if !confirm {
        error!(logger, "Cannot check without user confirmation");
        return Err(ErrorKind::Legacy(err_msg("operation aborded by the user")).into());
    }

    let config = args.value_of("config").unwrap();
    let config = Config::from_file(config)
        .context(ErrorKind::Legacy(err_msg("failed to check coordinator")))?;
    let admin = Admin::new(config.coordinator, logger.clone())
        .context(ErrorKind::Legacy(err_msg("failed to check coordinator")))?;
    let mut outcomes = Outcomes::new();

    // Check things.
    check_registry(&admin, &mut outcomes, interfaces, &logger)?;

    // Report results.
    if outcomes.has_errors() {
        error!(logger, "Coordinator data checks failed");
        return Err(ErrorKind::Legacy(err_msg("coordinator data checks failed")).into());
    }
    if outcomes.has_warnings() {
        warn!(logger, "Coordinator data checks passed with warnings");
        return Ok(());
    }
    info!(logger, "Coordinator data checks passed");
    Ok(())
}