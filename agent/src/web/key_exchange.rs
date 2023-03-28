use rocket::serde::{json::Json, Deserialize, Serialize};

use crate::{command_runner::run_command, constants::COMMAND_EXCHANGE_KEYS};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct HostInfo<'r> {
    host: &'r str,
    port: &'r str,
    secret: Option<&'r str>,
}

#[derive(Serialize, Debug)]
pub struct RegisterPublicKeyResponse {
    success: Option<bool>,
    error: Option<String>,
    code: Option<i32>,
}

#[post("/register-public-key", data = "<host_info>")]
pub fn register_public_key(host_info: Json<HostInfo<'_>>) -> Json<RegisterPublicKeyResponse> {
    let mut args: Vec<&str> = Vec::new();
    args.push(host_info.host);
    args.push(host_info.port);
    args.push(host_info.secret.unwrap_or(""));

    return match run_command(201, true, COMMAND_EXCHANGE_KEYS, args) {
        Ok(_) => Json(RegisterPublicKeyResponse {
            success: Some(true),
            error: None,
            code: None,
        }),
        Err(err) => Json(RegisterPublicKeyResponse {
            code: Some(err.code),
            success: None,
            error: Some(err.message),
        }),
    };
}
