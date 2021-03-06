use anyhow::Result;
use slog::Logger;
use structopt::StructOpt;
use uuid::Uuid;

mod approve;
mod disapprove;

// Command line options common to all action commands.
//
// This is included, possibly flattened, as arguments to leaf commands instead of additional
// options at the `action` level because we want to ensure the command is specified before
// these options.
//
// In other words we want `replictl action {approve, ...} $ACTION_ID`
// and not `replictl action $ACTION_ID {approve, ...}`.
// NOTE: this is not a docstring because StructOpt then uses it as the actions help.
#[derive(Debug, StructOpt)]
pub struct CommonOpt {
    /// ID of the action to operate on.
    #[structopt(env = "RCTL_ACTION")]
    pub action: Uuid,
}

/// Show and manage actions.
#[derive(Debug, StructOpt)]
pub enum Opt {
    /// Approve an action that is pending approval.
    Approve(CommonOpt),

    /// Disapprove (reject) an action that is pending approval.
    Disapprove(CommonOpt),
}

/// Execute the selected command.
pub async fn execute(logger: &Logger, opt: &crate::Opt, action_cmd: &Opt) -> Result<i32> {
    match &action_cmd {
        Opt::Approve(approve_opt) => approve::execute(logger, opt, approve_opt).await,
        Opt::Disapprove(disapprove_opt) => disapprove::execute(logger, opt, disapprove_opt).await,
    }
}
