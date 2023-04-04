use rocket::http::Status;
use tokio::process::Command;

use crate::constants::DEFAULTS;

#[delete("/transfers/<transfer_id>")]
pub async fn cancel_transfer(transfer_id: &str) -> Status {
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
