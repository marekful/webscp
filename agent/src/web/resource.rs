use rocket::{
    http::{CookieJar, Status},
    serde::{json::Json, Deserialize, Serialize},
    tokio::{task, time},
    State,
};
use std::{fs, time::Duration};
use urlencoding::encode;

use crate::client::Client;

use crate::{
    archive::{ArchiveItem, ArchiveWriter},
    command_runner::run_command_async,
    constants::{COMMAND_GET_REMOTE_RESOURCE, COMMAND_REMOTE_BEFORE_COPY, DEFAULTS},
    files_api::{FilesApi, Transfer},
    Files,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ResourceItem {
    source: String,
    destination: String,
    overwrite: bool,
    keep: bool,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CopyRequest {
    items: Vec<ResourceItem>,
    compress: bool,
    source_root: String,
}

#[derive(Serialize, Debug)]
pub struct ResourcesResponse {
    code: i32,
    resource: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct CopyResponse {
    code: i32,
    message: Option<String>,
}

#[get("/agents/<agent_id>/resources/<path>")]
pub async fn resources(
    agent_id: u32,
    path: &str,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<ResourcesResponse>) {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (agent, _) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(e) => {
            return (
                Status::new(e.http_code.unwrap_or(401)),
                Json(ResourcesResponse {
                    code: 902,
                    resource: None,
                    error: Some(e.message),
                }),
            )
        }
    };

    // create arguments for the 'get-remote-resource' command
    let mut args: Vec<&str> = Vec::new();
    let remote_user_id = agent.remote_user.id.clone().to_string();
    let path_encoded = encode(path);
    args.push(&agent.host);
    args.push(&agent.port);
    args.push(&remote_user_id);
    args.push(&agent.remote_user.token);
    args.push(&path_encoded);
    // execute command and send success or error response
    match run_command_async(202, true, false, COMMAND_GET_REMOTE_RESOURCE, args).await {
        Ok(output) => (
            Status::Ok,
            Json(ResourcesResponse {
                code: 0,
                resource: Some(output),
                error: None,
            }),
        ),
        Err(err) => (
            err.status,
            Json(ResourcesResponse {
                code: err.code,
                resource: None,
                error: Some(err.message),
            }),
        ),
    }
}

#[patch("/agents/<agent_id>/resources/<archive_name>", data = "<request>")]
pub async fn copy(
    agent_id: u32,
    archive_name: &str,
    request: Json<CopyRequest>,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<CopyResponse>) {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (agent, auth_token) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(e) => {
            return (
                Status::new(e.http_code.unwrap_or(401)),
                Json(CopyResponse {
                    code: 912,
                    message: None,
                }),
            )
        }
    };

    // create arguments for 'remote-before-copy' command
    let remote_user_id = &agent.remote_user.id.clone().to_string();
    let items_json = get_items_json(&request.items);
    let before_copy_args: Vec<&str> = vec![
        &agent.host,
        &agent.port,
        remote_user_id,
        &agent.remote_user.token,
        &items_json,
    ];

    // execute command
    let destination_root = match run_command_async(
        204,
        true,
        false,
        COMMAND_REMOTE_BEFORE_COPY,
        before_copy_args,
    )
    .await
    {
        Ok(root) => root,
        Err(err) => {
            // abort with error if copy pre-checks failed
            return (
                err.status,
                Json(CopyResponse {
                    code: err.code,
                    message: Some(err.message),
                }),
            );
        }
    };

    let transfer = Transfer {
        agent_id,
        host: agent.host,
        port: agent.port,
        transfer_id: archive_name.to_string(),
        local_path: String::from(&request.source_root),
        remote_path: destination_root,
        compress: request.compress,
        overwrite: request.items[0].overwrite,
        size: 0,
        rc_auth: auth_token.to_string(),
    };

    /*<alt:async execution of tar and scp> */
    // run remaining tasks asynchronously in a future
    let items_copy = request.items.to_vec();
    let _future = task::spawn(finish_upload_in_background(transfer, items_copy));

    /* The task has started execution at this point and
     * .await-ing it will be non-blocking. The task will
     * run to completion even without .await-ing it. */
    /*match future.await {
        Ok(fut) => {
            match fut {
                Ok(_) => {},
                Err(err) => {
                    return (
                        Status::ExpectationFailed,
                        Json(BeforeCopyResponse {
                            code: err.code,
                            message: Some(err.message),
                        }),
                    )
                }
            }
        }
        Err(err) => {
            return (
                Status::InternalServerError,
                Json(BeforeCopyResponse {
                    code: 245,
                    message: Some(err.to_string()),
                }),
            )
        }
    }*/
    /*<end:alt>*/

    // return success response
    (
        Status::Ok,
        Json(CopyResponse {
            code: 0,
            message: Some(archive_name.to_string()),
        }),
    )
}

pub struct FutureError {
    pub code: i32,
    pub message: String,
}

async fn finish_upload_in_background(
    mut transfer: Transfer,
    req_items: Vec<ResourceItem>,
) -> Result<(), FutureError> {
    // allow some time for the upload state poll to initialize
    time::sleep(Duration::from_millis(50)).await;

    // send progress update
    let files_api = FilesApi::new();
    let msg = match transfer.compress {
        true => "compressing",
        false => "archiving",
    };
    files_api
        .send_upload_status_update_async(&transfer, msg)
        .await;

    // create list of files to archive
    let mut items = Vec::new();
    for item in req_items.iter() {
        items.push(ArchiveItem {
            source: (item.source).parse().unwrap(),
            destination: (item.destination).parse().unwrap(),
        })
    }
    task::yield_now().await;

    // create archive of files
    let archive_path = &*format!(
        "{}{}.agent.tar.gz",
        DEFAULTS.temp_data_dir, transfer.transfer_id
    );
    let mut archive_writer =
        match ArchiveWriter::new(archive_path, transfer.compress, &transfer.local_path) {
            Ok(w) => w,
            Err(e) => {
                files_api
                    .send_upload_status_update_async(&transfer, &e.message)
                    .await;
                return Err(FutureError {
                    code: e.code,
                    message: e.message,
                });
            }
        };
    task::yield_now().await;

    if let Err(e) = archive_writer.crate_archive(items, &transfer).await {
        // send progress update and abort with error
        let err_msg = format!("{} (code:{})", e.message, e.code);
        files_api
            .send_upload_status_update_async(&transfer, &err_msg)
            .await;
        return Err(FutureError {
            code: e.code,
            message: e.message,
        });
    };
    task::yield_now().await;

    // ensure the gzip encoder has flushed
    drop(archive_writer);

    task::yield_now().await;

    transfer.size = fs::metadata(archive_path).unwrap().len();

    files_api
        .send_upload_status_update_async(&transfer, "starting upload")
        .await;

    // execute file upload
    match Client::remote_do_copy_async(&files_api, &transfer).await {
        Ok(_) => {
            files_api
                .send_upload_status_update_async(&transfer, "complete")
                .await;
            Ok(())
        }
        Err(e) => {
            let error = e.message;
            let err_msg = if e.code == 346 {
                format!("signal::interrupt::{}", error)
            } else {
                error.clone()
            };
            files_api
                .send_upload_status_update_async(&transfer, &err_msg)
                .await;
            Err(FutureError {
                code: e.code,
                message: error,
            })
        }
    }
}

fn get_items_json(items: &[ResourceItem]) -> String {
    // TODO: Use Display trait or otherwise improve this
    let mut json_str: Vec<String> = Vec::new();
    let mut first = true;
    json_str.push(String::from("'["));
    items.iter().for_each(|item| {
        let source = format!(
            "{{\"source\": \"{}\", \"destination\": \"{}\", \"overwrite\": {}, \"keep\": {}}}",
            item.source, item.destination, item.overwrite, item.keep
        );
        if first {
            first = false;
        } else {
            json_str.push(String::from(","));
        }
        json_str.push(source);
    });
    json_str.push(String::from("]'"));

    json_str.join("")
}
