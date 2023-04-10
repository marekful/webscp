#[path = "../cli/archive.rs"]
mod archive;
#[path = "../cli/client.rs"]
mod client;
#[path = "../cli/command.rs"]
mod command;
#[path = "../cli/command_runner.rs"]
mod command_runner;
#[path = "../cli/constants.rs"]
mod constants;
#[path = "../files_api.rs"]
mod files_api;

mod key_exchange;
mod miscellaneous;
mod remote_user;
mod resource;
mod temporary_access_token;
mod transfer;

#[macro_use]
extern crate rocket;

use crate::{
    files_api::FilesApi, key_exchange::*, miscellaneous::*, remote_user::*, resource::*,
    temporary_access_token::*, transfer::*,
};

pub struct Files {
    pub api: FilesApi,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let files = FilesApi::new();

    let api = "/api";
    let _rocket = rocket::build()
        .manage(Files { api: files })
        .mount(api, routes![get_temporary_access_token])
        .mount(api, routes![register_public_key])
        .mount(api, routes![get_remote_user])
        .mount(api, routes![ping])
        .mount(api, routes![resources])
        .mount(api, routes![copy])
        .mount(api, routes![version])
        .mount(api, routes![cancel_transfer])
        .launch()
        .await?;

    Ok(())
}

/*#[launch]
fn start_server() -> _ {
    let api = "/api";
    rocket::build()
        .mount(api, routes![register_public_key])
        .mount(api, routes![ping])
        .mount(api, routes![resources])
        .mount(api, routes![before_copy])
        .mount(api, routes![version])
}*/
