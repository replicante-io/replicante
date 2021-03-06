use failure::ResultExt;
use tokio::process::Command;

use crate::Conf;
use crate::ErrorKind;
use crate::Result;

/// Execute a command in podman unshare environment (enter the user ns but not others).
pub async fn unshare(conf: &Conf, command: Vec<&str>) -> Result<()> {
    let status = Command::new(&conf.podman)
        .arg("unshare")
        .args(command)
        .status()
        .await
        .with_context(|_| ErrorKind::podman_exec("unshare"))?;
    if !status.success() {
        let error = ErrorKind::podman_failed("unshare");
        return Err(error.into());
    }
    Ok(())
}
