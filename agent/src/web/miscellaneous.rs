use rocket::{
    http::CookieJar,
    serde::{json::Json, Serialize},
    State,
};

use crate::{
    command_runner::run_command_async,
    constants::{COMMAND_GET_REMOTE_VERSION, COMMAND_PING},
    Files,
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

#[get("/agents/<agent_id>/version")]
pub async fn version(
    agent_id: u32,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> Json<VersionResponse> {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (agent, _) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(e) => {
            return Json(VersionResponse {
                version: None,
                latency: None,
                error: Some("403 Forbidden".to_string()),
            })
        }
    };

    let mut version_ags: Vec<&str> = Vec::new();
    version_ags.push(&agent.host);
    version_ags.push(&agent.port);

    let version =
        match run_command_async(81, true, false, COMMAND_GET_REMOTE_VERSION, version_ags).await {
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
    ping_args.push(&agent.host);
    ping_args.push(&agent.port);

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

    Json(VersionResponse {
        latency: Some(ping.trim().to_string()),
        version: Some(version.trim().to_string()),
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
        Err(e) => {
            return Json(PingResponse {
                latency: None,
                error: Some("403 Forbidden".to_string()),
            })
        }
    };

    let mut args: Vec<&str> = Vec::new();
    args.push(&agent.host);
    args.push(&agent.port);

    return match run_command_async(71, true, false, COMMAND_PING, args).await {
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
