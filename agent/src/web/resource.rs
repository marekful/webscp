use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
    tokio::{task, time},
};
use std::{future::Future, time::Duration};

use crate::client::Client;

use crate::{
    archive::{ArchiveItem, ArchiveWriter},
    command_runner::run_command,
    constants::{COMMAND_GET_REMOTE_RESOURCE, COMMAND_REMOTE_BEFORE_COPY, DEFAULTS},
    fb_api::send_upload_status_update_async,
};
use crate::command_runner::run_command_async;

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
pub struct BeforeCopyRequest {
    items: Vec<ResourceItem>,
    //items: &'r str,
}

#[derive(Serialize, Debug)]
pub struct ResourcesResponse {
    code: i32,
    resource: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct BeforeCopyResponse {
    code: i32,
    message: Option<String>,
}

#[get("/resources/<host>/<port>/<path>")]
pub fn resources(host: &str, port: &str, path: &str) -> (Status, Json<ResourcesResponse>) {
    let mut args: Vec<&str> = Vec::new();
    args.push(host);
    args.push(port);
    args.push(path);

    return match run_command(202, true, false, COMMAND_GET_REMOTE_RESOURCE, args) {
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

#[post("/copy/<host>/<port>/<archive_name>", data = "<request>")]
pub async fn copy(
    host: &str,
    port: &str,
    archive_name: &str,
    request: Json<BeforeCopyRequest>,
) -> (Status, Json<BeforeCopyResponse>) {
    // create arguments for 'remote-before-copy' command
    let items_json = get_items_json(&request.items);
    let mut before_copy_args: Vec<&str> = Vec::new();
    before_copy_args.push(host);
    before_copy_args.push(port);
    before_copy_args.push(&items_json);
    // execute command
    match run_command_async(
        204,
        true,
        false,
        COMMAND_REMOTE_BEFORE_COPY,
        before_copy_args,
    ).await {
        Ok(_) => {}
        Err(err) => {
            return (
                err.status,
                Json(BeforeCopyResponse {
                    code: err.code,
                    message: Some(err.message),
                }),
            )
        }
    };

    /*<alt:async execution of tar and scp> */
    // run remaining tasks asynchronously in a future:
    let items_copy = request.items.to_vec();
    let _future = task::spawn(finish_upload_in_background(
        String::from(host),
        String::from(port),
        items_copy,
        String::from(archive_name),
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
        Json(BeforeCopyResponse {
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
    host: String,
    port: String,
    req_items: Vec<ResourceItem>,
    archive_name: String,
) -> impl Future<Output = Result<(), FutureError>> {
    async move {
        // allow some time for the upload state poll to initialize
        let _ = time::sleep(Duration::from_millis(50)).await;

        send_upload_status_update_async(&archive_name, "archiving").await;

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
        let mut archive_writer = match ArchiveWriter::new(archive_path, false) {
            Ok(w) => w,
            Err(e) => {
                send_upload_status_update_async(&archive_name, &e.message).await;
                return Err(FutureError {
                    code: e.code,
                    message: e.message,
                });
            }
        };
        task::yield_now().await;

        if let Err(e) = archive_writer.crate_archive(items).await {
            send_upload_status_update_async(&archive_name, &e.message).await;
            return Err(FutureError {
                code: e.code,
                message: e.message,
            });
        };
        task::yield_now().await;

        send_upload_status_update_async(&archive_name, "uploading").await;

        // --->ORIG execute command
        // create arguments for 'remote-do-copy' command
        /*let mut do_copy_args: Vec<&str> = Vec::new();
        do_copy_args.push(&host);
        do_copy_args.push(&port);
        do_copy_args.push(&archive_name);

        return match run_command(203, true, false, COMMAND_REMOTE_DO_COPY, do_copy_args) {
            Ok(_) => {
                send_upload_status_update_async(&archive_name, "complete").await;
                Ok(())
            }
            Err(e) => {
                send_upload_status_update_async(&archive_name, &e.message).await;
                Err(FutureError {
                    code: e.code,
                    message: e.message,
                })
            }
        };*/
        // <---ORIG

        match Client::remote_do_copy_async(&host, &port, &archive_name).await {
            Ok(_) => {
                send_upload_status_update_async(&archive_name, "complete").await;
                Ok(())
            }
            Err(e) => {
                let err_msg;
                if e.code == 346 {
                    err_msg = format!("signal::interrupt::{}", e.message);
                } else {
                    err_msg = format!("error::{}", e.message);
                }
                send_upload_status_update_async(&archive_name, &err_msg).await;
                return Err(FutureError {
                    code: e.code,
                    message: e.message,
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
