use crate::constants::DEFAULTS;
use rocket::{
    http::Status,
    serde::{json::Json, Serialize},
};
use tokio::process::Command;

#[derive(Serialize, Debug)]
pub struct CancelTransferResponse {
    code: i32,
    success: bool,
    message: Option<String>,
}

#[delete("/transfers/<transfer_id>")]
pub async fn cancel_transfer(transfer_id: &str) -> (Status, Json<CancelTransferResponse>) {
    // create argument list for uploader script
    let mut script_args: Vec<&str> = Vec::new();
    script_args.push(DEFAULTS.cancel_transfer_script_path);
    script_args.push(&transfer_id);

    // setup and execute command
    let mut cmd = Command::new("bash");
    let child = cmd.args(script_args).spawn();
    let result = child.unwrap().wait().await;

    if result.is_err() {
        return (
            Status::InternalServerError,
            Json(CancelTransferResponse {
                code: 119,
                success: false,
                message: None,
            }),
        );
    }

    let result = result.unwrap();
    let code = result.code().unwrap();
    if code != 0 {
        return (
            Status::NotFound,
            Json(CancelTransferResponse {
                code: 120,
                success: false,
                message: Some(format!("code: {code}")),
            }),
        );
    }

    (
        Status::Ok,
        Json(CancelTransferResponse {
            code: 0,
            success: true,
            message: None,
        }),
    )
}
