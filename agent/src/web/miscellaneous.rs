use rocket::{
    http::CookieJar,
    serde::{
        json::{serde_json, Json},
        Serialize,
    },
    State,
};
use serde::Deserialize;

use crate::{
    command_runner::run_command_async,
    constants::{COMMAND_GET_REMOTE_VERSION, COMMAND_PING},
    Files,
};

#[derive(Serialize, Debug)]
pub struct VersionResponse {
    version: Option<Version>,
    error: Option<String>,
    latency: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Version {
    files: String,
    agent: String,
}

#[derive(Serialize, Debug)]
pub struct PingResponse {
    latency: Option<String>,
    error: Option<String>,
}

#[get("/agents/<agent_id>/version")]
pub async fn version(
    agent_id: u32,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> Json<VersionResponse> {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (agent, _) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(_) => {
            return Json(VersionResponse {
                version: None,
                latency: None,
                error: Some("403 Forbidden".to_string()),
            })
        }
    };

    // create arguments for the 'get-remote-version' command
    let version_args: Vec<&str> = vec![&agent.host, &agent.port];

    // execute 'get-remote-version' command
    let version_str =
        match run_command_async(81, true, false, COMMAND_GET_REMOTE_VERSION, version_args).await {
            Ok(version) => version,
            Err(err) => {
                return Json(VersionResponse {
                    version: None,
                    latency: None,
                    error: Some(err.message),
                })
            }
        };

    // create arguments for the 'ping' command
    let ping_args: Vec<&str> = vec![&agent.host, &agent.port];

    // execute 'ping' command
    let ping = match run_command_async(91, true, false, COMMAND_PING, ping_args).await {
        Ok(ping) => ping,
        Err(err) => {
            return Json(VersionResponse {
                version: None,
                latency: None,
                error: Some(err.message),
            })
        }
    };

    // parse version json result
    let deserialized_result = serde_json::from_str(&version_str);
    if deserialized_result.is_err() {
        return Json(VersionResponse {
            version: None,
            latency: None,
            error: Some(format!(
                "parse error: {} -- {}",
                deserialized_result.unwrap_err(),
                version_str
            )),
        });
    }
    let version: Version = deserialized_result.unwrap();

    Json(VersionResponse {
        latency: Some(ping.trim().to_string()),
        version: Some(version),
        error: None,
    })
}

#[get("/agents/<agent_id>/ping")]
pub async fn ping(
    agent_id: u32,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> Json<PingResponse> {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (agent, _) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(_) => {
            return Json(PingResponse {
                latency: None,
                error: Some("403 Forbidden".to_string()),
            })
        }
    };

    let args: Vec<&str> = vec![&agent.host, &agent.port];
    match run_command_async(71, true, false, COMMAND_PING, args).await {
        Ok(output) => Json(PingResponse {
            latency: Some(output.trim().to_string()),
            error: None,
        }),
        Err(err) => Json(PingResponse {
            latency: None,
            error: Some(err.message),
        }),
    }
}
