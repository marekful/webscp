use std::process::exit;

use crate::{client::*, constants::*};

pub fn command_exchange_keys(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!(
            "Usage: cli {} <host> <port> <agent_secret>",
            COMMAND_EXCHANGE_KEYS
        );
        exit(136);
    }
    let secret = &args[4];
    client.exchange_keys(secret);
}

pub fn command_get_remote_user(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 7 {
        eprintln!(
            "Usage: cli {} <host> <port> <username> <password> <access_token>",
            COMMAND_GET_REMOTE_USER
        );
        exit(140);
    }
    let user_name = &args[4];
    let password = &args[5];
    let token = &args[6];
    let exit_code = client.get_remote_user(user_name, password, token);

    if exit_code != 0 {
        exit(exit_code);
    }
}

pub fn command_get_remote_resource(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 6 {
        eprintln!(
            "Usage: cli {} <host> <port> <user_id> <path>",
            COMMAND_GET_REMOTE_RESOURCE
        );
        exit(137);
    }
    let user_id: u32 = args[4].parse().unwrap_or(0);
    let path = &args[5];

    client.get_remote_resource(user_id, path);
}

pub fn command_get_local_resource(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 4 {
        eprintln!(
            "Usage: client {} <user_id> <path>",
            COMMAND_GET_LOCAL_RESOURCE
        );
        exit(138);
    }
    let user_id: u32 = args[2].parse().unwrap_or(0);
    let path = &args[3];

    match client.files_api.get_local_resource(user_id, path) {
        Ok(resources_result) => {
            print!("{resources_result}");
        }
        Err(e) => {
            eprint!("{}", e.message);
            exit(e.code);
        }
    }
}

pub fn command_get_local_user(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 4 {
        eprintln!(
            "Usage: client {} <username> <password>",
            COMMAND_GET_LOCAL_USER
        );
        exit(139);
    }
    let user_name = &args[2];
    let password = &args[3];

    match client.files_api.get_local_user(user_name, password) {
        Ok(user_response) => {
            print!("{user_response}");
        }
        Err(e) => {
            eprint!("{}", e.message);
            exit(e.code);
        }
    }
}

pub fn command_get_remote_version(client: Client, _: Option<Vec<String>>) {
    client.get_remote_version();
}

pub fn command_ping(client: Client, _: Option<Vec<String>>) {
    client.ping();
}

pub fn command_get_local_version(client: Client, _: Option<Vec<String>>) {
    const AGENT_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    let agent_version = AGENT_VERSION.unwrap_or("unknown").to_string();
    let fb_version = client.files_api.get_version();

    println!("{} / {}", agent_version, fb_version);
}

pub fn command_remote_before_copy(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 6 {
        eprintln!(
            "Usage: cli {} <host> <port> <user_id> <items>",
            COMMAND_REMOTE_BEFORE_COPY
        );
        exit(140);
    }
    let user_id: u32 = args[4].parse().unwrap_or(0);
    let items = &args[5];
    let exit_code = client.remote_before_copy(user_id, items);

    if exit_code != 0 {
        exit(exit_code);
    }
}

pub fn command_local_before_copy(client: Client<'_>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 4 {
        eprintln!("Usage: cli {} <user_id> <items>", COMMAND_LOCAL_BEFORE_COPY);
        exit(142);
    }

    let user_id: u32 = args[2].parse().unwrap_or(0);
    let items = String::from(&args[3]);

    match client.files_api.local_before_copy(user_id, items) {
        Ok(response) => {
            print!("{}", response.trim().to_string());
        }
        Err(e) => {
            eprintln!("{}", e.message.trim());
            exit(e.code);
        }
    }
}
