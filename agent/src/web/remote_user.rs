use rocket::{
    http::{CookieJar, Status},
    serde::{
        json::{serde_json, Json},
        Deserialize, Serialize,
    },
    State,
};

use crate::{
    command_runner::run_command_async,
    constants::{COMMAND_GET_REMOTE_USER, COMMAND_GET_TOKEN_USER},
    Files,
};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GetRemoteUserRequest<'r> {
    name: &'r str,
    password: &'r str,
}

#[derive(Serialize, Debug)]
pub struct GetRemoteUserResponse {
    code: i32,
    id: Option<u32>,
    token: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GetTokenUserRequest<'r> {
    access_token: &'r str,
}

#[derive(Serialize, Debug)]
pub struct GetTokenUserResponse {
    code: i32,
    id: Option<u32>,
    name: Option<String>,
    branding: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RemoteUser {
    id: u32,
    token: String,
}

#[derive(Deserialize, Debug)]
struct TokenUser {
    id: u32,
    name: String,
    branding: String,
}

#[post(
    "/users/<user_id>/connections/<host>/<port>/token-user",
    data = "<request>"
)]
pub async fn get_token_user(
    user_id: u32,
    host: &str,
    port: &str,
    request: Json<GetTokenUserRequest<'_>>,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<GetTokenUserResponse>) {
    // check user session
    let res = files
        .api
        .check_user_auth(user_id, cookies.get("rc_auth"))
        .await;
    if let Err(err) = res {
        return (
            Status::Unauthorized,
            Json(GetTokenUserResponse {
                code: err.code,
                id: None,
                name: None,
                branding: None,
                error: Some(err.message),
            }),
        );
    }
    // create argument list for the get-remote-user-name command
    let args: Vec<&str> = vec![host, port, request.access_token];

    // execute command
    let result = run_command_async(274, true, false, COMMAND_GET_TOKEN_USER, args).await;

    // return error response if failed to execute command
    if let Err(err) = result {
        return (
            err.status,
            Json(GetTokenUserResponse {
                code: err.code,
                id: None,
                name: None,
                branding: None,
                error: Some(err.message),
            }),
        );
    }

    // attempt to parse token user from response
    let result_str = result.unwrap_or("".to_string());
    let deserialized_result = serde_json::from_str(&result_str);

    // return error response if no remote user could be parsed
    if let Err(err) = deserialized_result {
        return (
            Status::InternalServerError,
            Json(GetTokenUserResponse {
                code: 625,
                id: None,
                name: None,
                branding: None,
                error: Some(err.to_string()),
            }),
        );
    }
    let token_user: TokenUser = deserialized_result.unwrap();

    (
        Status::Ok,
        Json(GetTokenUserResponse {
            code: 0,
            id: Some(token_user.id),
            name: Some(token_user.name),
            branding: Some(token_user.branding),
            error: None,
        }),
    )
}

#[post("/users/<user_id>/connections/<host>/<port>/login", data = "<request>")]
pub async fn get_remote_user(
    user_id: u32,
    host: &str,
    port: &str,
    request: Json<GetRemoteUserRequest<'_>>,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<GetRemoteUserResponse>) {
    // check user session
    let res = files
        .api
        .check_user_auth(user_id, cookies.get("rc_auth"))
        .await;
    if let Err(err) = res {
        return (
            Status::Unauthorized,
            Json(GetRemoteUserResponse {
                code: err.code,
                id: None,
                token: None,
                error: Some(err.message),
            }),
        );
    }

    // create argument list for the get-remote-user command
    let args: Vec<&str> = vec![host, port, request.name, request.password];

    // execute command
    let result = run_command_async(274, true, false, COMMAND_GET_REMOTE_USER, args).await;

    // return error response if failed to execute command
    if let Err(err) = result {
        return (
            err.status,
            Json(GetRemoteUserResponse {
                code: err.code,
                id: None,
                token: None,
                error: Some(err.message),
            }),
        );
    }

    // attempt to parse remote user from response
    let result_str = result.unwrap_or("".to_string());
    let deserialized_result = serde_json::from_str(&result_str);

    // return error response if no remote user could be parsed
    if deserialized_result.is_err() {
        return (
            Status::InternalServerError,
            Json(GetRemoteUserResponse {
                code: 325,
                id: None,
                token: None,
                error: None,
            }),
        );
    }
    let remote_user: RemoteUser = deserialized_result.unwrap();

    // return success response
    (
        Status::Ok,
        Json(GetRemoteUserResponse {
            code: 0,
            id: Some(remote_user.id),
            token: Some(remote_user.token),
            error: None,
        }),
    )
}
