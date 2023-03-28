use rocket::serde::{json::Json, Serialize};

use crate::{
    command_runner::run_command,
    constants::{COMMAND_GET_REMOTE_VERSION, COMMAND_PING},
};

#[derive(Serialize, Debug)]
pub struct VersionResponse {
    version: Option<String>,
    error: Option<String>,
    latency: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct PingResponse {
    latency: Option<String>,
    error: Option<String>,
}

#[get("/version/<host>/<port>")]
pub fn version(host: &str, port: &str) -> Json<VersionResponse> {
    let mut version_ags: Vec<&str> = Vec::new();
    version_ags.push(host);
    version_ags.push(port);

    let version = match run_command(81, true, COMMAND_GET_REMOTE_VERSION, version_ags) {
        Ok(version) => version,
        Err(err) => {
            return Json(VersionResponse {
                version: None,
                latency: None,
                error: Some(err.message),
            })
        }
    };

    let mut ping_args: Vec<&str> = Vec::new();
    ping_args.push(host);
    ping_args.push(port);

    let ping = match run_command(91, true, COMMAND_PING, ping_args) {
        Ok(ping) => ping,
        Err(err) => {
            return Json(VersionResponse {
                version: None,
                latency: None,
                error: Some(err.message),
            })
        }
    };

    Json(VersionResponse {
        latency: Some(ping.trim().to_string()),
        version: Some(version.trim().to_string()),
        error: None,
    })
}

#[get("/ping/<host>/<port>")]
pub fn ping(host: &str, port: &str) -> Json<PingResponse> {
    let mut args: Vec<&str> = Vec::new();
    args.push(host);
    args.push(port);

    return match run_command(71, true, COMMAND_PING, args) {
        Ok(output) => Json(PingResponse {
            latency: Some(output.trim().to_string()),
            error: None,
        }),
        Err(err) => Json(PingResponse {
            latency: None,
            error: Some(err.message),
        }),
    };
}
