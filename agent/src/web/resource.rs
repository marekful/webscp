use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
};

use crate::{
    archive::{ArchiveItem, ArchiveWriter},
    command_runner::run_command,
    constants::{
        COMMAND_GET_REMOTE_RESOURCE, COMMAND_REMOTE_BEFORE_COPY, COMMAND_REMOTE_DO_COPY, DEFAULTS,
    },
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

    return match run_command(202, true, COMMAND_GET_REMOTE_RESOURCE, args) {
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
pub fn copy(
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
    match run_command(204, true, COMMAND_REMOTE_BEFORE_COPY, before_copy_args) {
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

    /*<alt:async execution of tar and scp>*/

    /*<alt:synchronous execution of tar and scp> */
    // create list of files to archive
    let mut items = Vec::new();
    for item in request.items.iter() {
        items.push(ArchiveItem {
            source: (item.source).parse().unwrap(),
            destination: (item.destination).parse().unwrap(),
        })
    }

    // create archive of files
    let archive_path = &*format!("{}{archive_name}.agent.tar.gz", DEFAULTS.temp_data_dir);
    let mut archive_writer = match ArchiveWriter::new(archive_path, false) {
        Ok(w) => w,
        Err(e) => {
            return (
                Status::ExpectationFailed,
                Json(BeforeCopyResponse {
                    code: e.code,
                    message: Some(e.message),
                }),
            )
        }
    };
    match archive_writer.crate_archive(items) {
        Ok(_) => {}
        Err(e) => {
            return (
                Status::ExpectationFailed,
                Json(BeforeCopyResponse {
                    code: e.code,
                    message: Some(e.message),
                }),
            )
        }
    }

    // create arguments for 'remote-do-copy' command
    let mut do_copy_args: Vec<&str> = Vec::new();
    do_copy_args.push(host);
    do_copy_args.push(port);
    do_copy_args.push(archive_name);
    // execute command
    let result = match run_command(203, true, COMMAND_REMOTE_DO_COPY, do_copy_args) {
        Ok(output) => output,
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
    /*<end:alt>*/

    // return success response
    (
        Status::Ok,
        Json(BeforeCopyResponse {
            code: 0,
            message: Some(result),
        }),
    )
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
