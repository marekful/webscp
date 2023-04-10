use rocket::{
    http::Status,
    serde::{
        json::{serde_json, Json, Value},
        Deserialize, Serialize,
    },
};

use crate::{command_runner::run_command_async, constants::COMMAND_GET_REMOTE_USER};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GetRemoteUserRequest<'r> {
    name: &'r str,
    password: &'r str,
    access_token: &'r str,
}
#[derive(Serialize, Debug)]
pub struct GetRemoteUserResponse {
    code: i32,
    id: Option<u32>,
    root: Option<String>,
    error: Option<String>,
}

#[post("/get-remote-user/<host>/<port>", data = "<request>")]
pub async fn get_remote_user(
    host: &str,
    port: &str,
    request: Json<GetRemoteUserRequest<'_>>,
) -> (Status, Json<GetRemoteUserResponse>) {
    // create argument list for the get-remote-user command
    let mut get_user_args: Vec<&str> = Vec::new();
    get_user_args.push(host);
    get_user_args.push(port);
    get_user_args.push(request.name);
    get_user_args.push(request.password);
    get_user_args.push(request.access_token);

    // execute command
    let result = run_command_async(274, true, false, COMMAND_GET_REMOTE_USER, get_user_args).await;

    // return error response if failed to execute command
    if result.is_err() {
        let err = result.unwrap_err();
        return (
            err.status,
            Json(GetRemoteUserResponse {
                code: err.code,
                id: None,
                root: None,
                error: Some(err.message),
            }),
        );
    }

    // attempt to parse remote user's ID from response
    let result_str = result.unwrap_or("".to_string());
    let p: Value = serde_json::from_str(&result_str).unwrap();
    let user_id = p["id"].as_u64().unwrap_or(0);
    let user_root = p["root"].as_str().unwrap_or("");

    // return error response if no ID could be parsed
    if user_id == 0 {
        return (
            Status::InternalServerError,
            Json(GetRemoteUserResponse {
                code: 327,
                id: None,
                root: None,
                error: None,
            }),
        );
    }

    // return success response
    (
        Status::Ok,
        Json(GetRemoteUserResponse {
            code: 0,
            id: Some(user_id as u32),
            root: Some(user_root.to_string()),
            error: None,
        }),
    )
}
