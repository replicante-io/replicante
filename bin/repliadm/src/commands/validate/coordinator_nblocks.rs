use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use slog::info;

use replicante_util_failure::format_fail;

pub const COMMAND: &str = "coordinator-nblocks";

use crate::outcome::Error;
use crate::outcome::Outcomes;
use crate::utils::coordinator_admin;
use crate::Interfaces;
use crate::Result;

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(COMMAND).about("Validate coordinator non-blocking locks")
}

pub fn run<'a>(args: &ArgMatches<'a>, interfaces: &Interfaces) -> Result<Outcomes> {
    let logger = interfaces.logger();
    info!(logger, "Checking held non-blocking locks");

    let admin = coordinator_admin(args, logger.clone())?;
    let mut outcomes = Outcomes::new();
    let mut tracker = interfaces.progress("Processed more non-blocking locks");

    for lock in admin.non_blocking_locks() {
        if let Err(error) = lock {
            let error = format_fail(&error);
            outcomes.error(Error::GenericError(error));
        }
        tracker.track();
    }

    Ok(outcomes)
}
