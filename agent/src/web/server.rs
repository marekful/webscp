#[path = "../cli/archive.rs"]
mod archive;
#[path = "../cli/command_runner.rs"]
mod command_runner;
#[path = "../cli/constants.rs"]
mod constants;

mod key_exchange;
mod miscellaneous;
mod resource;

#[macro_use]
extern crate rocket;
use crate::{key_exchange::*, miscellaneous::*, resource::*};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let api = "/api";
    let _rocket = rocket::build()
        .mount(api, routes![register_public_key])
        .mount(api, routes![ping])
        .mount(api, routes![resources])
        .mount(api, routes![copy])
        .mount(api, routes![version])
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
