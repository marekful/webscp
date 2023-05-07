use rocket::{
    http::{CookieJar, Status},
    serde::{json::Json, Deserialize, Serialize},
    State,
};

use crate::{command_runner::run_command_async, constants::COMMAND_EXCHANGE_KEYS, Files};

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

#[post("/users/<user_id>/connections", data = "<host_info>")]
pub async fn register_public_key(
    user_id: u32,
    host_info: Json<HostInfo<'_>>,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<RegisterPublicKeyResponse>) {
    // check user session
    let res = files
        .api
        .check_user_auth(user_id, cookies.get("rc_auth"))
        .await;
    if res.is_err() {
        let err = res.unwrap_err();
        return (
            Status::Unauthorized,
            Json(RegisterPublicKeyResponse {
                code: Some(err.code),
                success: None,
                error: Some(err.message),
            }),
        );
    }

    let args: Vec<&str> = vec![
        host_info.host,
        host_info.port,
        host_info.secret.unwrap_or(""),
    ];

    match run_command_async(201, true, false, COMMAND_EXCHANGE_KEYS, args).await {
        Ok(_) => (
            Status::Ok,
            Json(RegisterPublicKeyResponse {
                success: Some(true),
                error: None,
                code: None,
            }),
        ),
        Err(err) => (
            err.status,
            Json(RegisterPublicKeyResponse {
                code: Some(err.code),
                success: None,
                error: Some(err.message),
            }),
        ),
    }
}
