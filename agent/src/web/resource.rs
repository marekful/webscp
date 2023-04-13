use rocket::{
    http::{CookieJar, Status},
    serde::{json::Json, Deserialize, Serialize},
    tokio::{task, time},
    State,
};
use std::{future::Future, time::Duration};

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
    rename: bool,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CopyRequest {
    items: Vec<ResourceItem>,
    source_root: String,
    destination_root: String,
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

#[get("/resources/<agent_id>/<path>")]
pub async fn resources(
    agent_id: u32,
    path: &str,
    files: &State<Files>,
    cookies: &CookieJar<'_>,
) -> (Status, Json<ResourcesResponse>) {
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

    let mut args: Vec<&str> = Vec::new();
    let remote_user_id = agent.remote_user.id.clone().to_string();
    args.push(&agent.host);
    args.push(&agent.port);
    args.push(&remote_user_id);
    args.push(path);

    return match run_command_async(202, true, false, COMMAND_GET_REMOTE_RESOURCE, args).await {
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
    };
}

#[post("/copy/<agent_id>/<archive_name>", data = "<request>")]
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
    let mut before_copy_args: Vec<&str> = Vec::new();
    before_copy_args.push(&agent.host);
    before_copy_args.push(&agent.port);
    before_copy_args.push(remote_user_id);
    before_copy_args.push(&items_json);
    // execute command
    match run_command_async(
        204,
        true,
        false,
        COMMAND_REMOTE_BEFORE_COPY,
        before_copy_args,
    )
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return (
                err.status,
                Json(CopyResponse {
                    code: err.code,
                    message: Some(err.message),
                }),
            )
        }
    };

    let transfer = Transfer {
        agent_id,
        transfer_id: archive_name.to_string(),
        rc_auth: auth_token.to_string(),
    };

    /*<alt:async execution of tar and scp> */
    // run remaining tasks asynchronously in a future
    let items_copy = request.items.to_vec();
    let _future = task::spawn(finish_upload_in_background(
        transfer,
        String::from(&agent.host),
        String::from(&agent.port.to_string()),
        items_copy,
        String::from(archive_name),
        String::from(&request.source_root),
        String::from(&request.destination_root),
    ));

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

fn finish_upload_in_background(
    transfer: Transfer,
    host: String,
    port: String,
    req_items: Vec<ResourceItem>,
    archive_name: String,
    local_path: String,
    remote_path: String,
) -> impl Future<Output = Result<(), FutureError>> + 'static {
    async move {
        // allow some time for the upload state poll to initialize
        let _ = time::sleep(Duration::from_millis(50)).await;

        let files_api = FilesApi::new();

        files_api
            .send_upload_status_update_async(&transfer, "archiving")
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
        let archive_path = &*format!("{}{archive_name}.agent.tar.gz", DEFAULTS.temp_data_dir);
        let mut archive_writer = match ArchiveWriter::new(archive_path, false, &local_path) {
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

        if let Err(e) = archive_writer.crate_archive(items).await {
            let err_msg = format!("{} code:({})", e.message, e.code);
            files_api
                .send_upload_status_update_async(&transfer, &err_msg)
                .await;
            return Err(FutureError {
                code: e.code,
                message: e.message,
            });
        };
        task::yield_now().await;

        files_api
            .send_upload_status_update_async(&transfer, "uploading")
            .await;

        // --->ORIG execute command
        // create arguments for 'remote-do-copy' command
        /*let mut do_copy_args: Vec<&str> = Vec::new();
        do_copy_args.push(&host);
        do_copy_args.push(&port);
        do_copy_args.push(&archive_name);

        return match run_command(203, true, false, COMMAND_REMOTE_DO_COPY, do_copy_args) {
            Ok(_) => {
                files_api.send_upload_status_update_async(&archive_name, "complete").await;
                Ok(())
            }
            Err(e) => {
                files_api.send_upload_status_update_async(&archive_name, &e.message).await;
                Err(FutureError {
                    code: e.code,
                    message: e.message,
                })
            }
        };*/
        // <---ORIG

        match Client::remote_do_copy_async(
            &files_api,
            &transfer,
            &host,
            &port,
            &archive_name,
            &remote_path,
        )
        .await
        {
            Ok(_) => {
                files_api
                    .send_upload_status_update_async(&transfer, "complete")
                    .await;
                Ok(())
            }
            Err(e) => {
                let err_msg;
                let error = e.message;
                if e.code == 346 {
                    err_msg = format!("signal::interrupt::{}", error);
                } else {
                    err_msg = error.clone();
                }
                files_api
                    .send_upload_status_update_async(&transfer, &err_msg)
                    .await;
                return Err(FutureError {
                    code: e.code,
                    message: error,
                });
            }
        }
    }
}

fn get_items_json(items: &Vec<ResourceItem>) -> String {
    // TODO: Use Display trait or otherwise improve this
    let mut json_str: Vec<String> = Vec::new();
    let mut first = true;
    json_str.push(String::from("'["));
    items.iter().for_each(|item| {
        let source = format!(
            "{{\"source\": \"{}\", \"destination\": \"{}\", \"override\": {}, \"rename\": {}}}",
            item.source, item.destination, item.overwrite, item.rename
        );
        if first == true {
            first = false;
        } else {
            json_str.push(String::from(","));
        }
        json_str.push(source);
    });
    json_str.push(String::from("]'"));

    json_str.join("")
}
