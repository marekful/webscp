use rocket::{
    http::{CookieJar, Status},
    serde::{json::Json, Serialize},
    State,
};
use rustfmt::config::NewlineStyle::Unix;
use sha256::digest;
use std::{
    future::Future,
    ops::Add,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{task, time::Instant};

use crate::{
    client::Client,
    command_runner::run_command_async,
    constants::DEFAULTS,
    Files,
};

#[derive(Serialize, Debug)]
pub struct TemporaryAccessTokenResponse {
    code: i32,
    token: Option<String>,
    valid_until: Option<u64>,
    error: Option<String>,
}

#[get("/users/<user_id>/temporary-access-token")]
pub async fn get_temporary_access_token(
    user_id: u32,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<TemporaryAccessTokenResponse>) {
    // check user session
    let user = match files
        .api
        .get_auth_user(user_id, cookies.get("rc_auth"))
        .await
    {
        Ok(u) => u,
        Err(_) => {
            return (
                Status::Unauthorized,
                Json(TemporaryAccessTokenResponse {
                    code: 922,
                    token: None,
                    valid_until: None,
                    error: None,
                }),
            );
        }
    };

    // create arguments for the generate temporary access token command
    let key_id = Client::random_hex();
    let user_id = user.id.to_string();
    let mut args: Vec<&str> = Vec::new();
    args.push(DEFAULTS.generate_key_pair_script_path);
    args.push(&key_id);
    args.push(&user_id);
    args.push(&user.username);
    args.push(&user.scope);

    // execute command
    let token = match run_command_async(280, false, false, "bash", args).await {
        Ok(t) => t,
        Err(err) => {
            return (
                err.status,
                Json(TemporaryAccessTokenResponse {
                    code: err.code,
                    token: None,
                    error: Some(err.message),
                    valid_until: None,
                }),
            )
        }
    };

    // schedule temporary access token revocation
    let (valid_until_timestamp, valid_until_instant) = valid_until(300);
    let token_hash = digest(token.clone());
    let _future = task::spawn(revoke_temporary_access_token(
        key_id,
        token_hash.clone(),
        valid_until_instant,
    ));

    // send token in response
    (
        Status::Ok,
        Json(TemporaryAccessTokenResponse {
            code: 0,
            token: Some(token),
            valid_until: Some(valid_until_timestamp),
            error: None,
        }),
    )
}

fn valid_until(seconds: u64) -> (u64, Instant) {
    let instant = Instant::now().add(Duration::from_secs(seconds));
    let systime = SystemTime::now().add(Duration::from_secs(seconds));
    let timestamp = systime.duration_since(UNIX_EPOCH).unwrap().as_secs();

    (timestamp, instant)
}

fn revoke_temporary_access_token(
    token_id: String,
    token_hash: String,
    token_expires: Instant,
) -> impl Future<Output = Result<(), ()>> + 'static {
    async move {
        let one_sec = Duration::from_secs(1);
        let lock_file_path = format!("{}/{}", DEFAULTS.ssh_dir_path, token_hash);

        // create arguments for the revoke key command
        let mut args: Vec<&str> = Vec::new();
        args.push(DEFAULTS.revoke_key_pair_script_path);
        args.push(&token_id);

        // initialize command
        let revoke_key = run_command_async(281, false, false, "bash", args);

        // create arguments for the remove lock file command
        let mut args: Vec<&str> = Vec::new();
        args.push("-f");
        args.push(&lock_file_path);

        // initialize command
        let remove_lock_file = run_command_async(282, false, false, "rm", args);

        // loop-wait until timeout or lock file removed (by registered client)
        loop {
            tokio::time::sleep(one_sec).await;

            if !Path::new(&lock_file_path).exists() {
                // proceed to revoke key
                break;
            }

            if token_expires < Instant::now() {
                // execute remove lock file command
                let _ = remove_lock_file.await;

                // proceed to revoke key
                break;
            }
        }

        // execute revoke key command
        let _ = revoke_key.await;

        Ok(())
    }
}
