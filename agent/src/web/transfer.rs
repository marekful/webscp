use rocket::{
    http::{CookieJar, Status},
    State,
};
use tokio::process::Command;

use crate::{constants::DEFAULTS, Files};

#[delete("/agents/<agent_id>/transfers/<transfer_id>")]
pub async fn cancel_transfer(
    agent_id: u32,
    transfer_id: &str,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> Status {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (_, _) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(_) => {
            return Status::Forbidden;
        }
    };

    // create argument list for uploader script
    let mut script_args: Vec<&str> = Vec::new();
    script_args.push(DEFAULTS.cancel_transfer_script_path);
    script_args.push(&transfer_id);

    // setup and execute command
    let mut cmd = Command::new("bash");
    let child = cmd.args(script_args).spawn();
    let result = child.unwrap().wait().await;

    // respond with error on command error
    if result.is_err() {
        return Status::InternalServerError;
    }

    // respond with not found on non-zero exit code
    let result = result.unwrap();
    let code = result.code().unwrap();
    if code != 0 {
        return Status::NotFound;
    }

    // success response
    Status::Ok
}
