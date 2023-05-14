#[path = "../cli/archive.rs"]
pub mod archive;
#[path = "../cli/client.rs"]
pub mod client;
#[path = "../cli/command.rs"]
pub mod command;
#[path = "../cli/command_runner.rs"]
mod command_runner;
#[path = "../cli/constants.rs"]
pub mod constants;
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
    temporary_access_token::*, transfer::cancel_transfer,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// CancelTransferRequests holds a mutable reference to a list
/// shared across threads where new transfers register a 'cancel
/// requested' flag for themselves. This flag is monitored during
/// various task execution phases of each transfer so it can
/// initiate a self abort whenever the flags state is flipped.
/// A transfer's 'cancel requested' flag is is set via a user
/// initiated request at an arbitrary phase of the transfer.
pub struct CancelTransferRequests {
    pub transfers: Arc<Mutex<HashMap<String, Arc<Mutex<bool>>>>>,
}

pub struct Files {
    pub api: FilesApi,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let transfers: Arc<Mutex<HashMap<String, Arc<Mutex<bool>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let files = FilesApi::new();

    let api = "/api";
    let _rocket = rocket::build()
        .manage(Files { api: files })
        .manage(CancelTransferRequests { transfers })
        .mount(api, routes![get_temporary_access_token])
        .mount(api, routes![register_public_key])
        .mount(api, routes![get_token_user])
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
