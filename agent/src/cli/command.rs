use std::process::exit;

use crate::{client::*, constants::*};

pub fn command_exchange_keys(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: client exchange-keys <host> <port> <agent_secret>");
        exit(136);
    }
    let secret = &args[4];
    client.exchange_keys(secret);
}

pub fn command_get_remote_resource(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: client get-remote-resource <host> <port> <path>");
        exit(137);
    }
    let path = &args[4];
    client.get_remote_resource(path);
}

pub fn command_get_local_resource(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 3 {
        eprintln!("Usage: client {COMMAND_GET_LOCAL_RESOURCE} <path>");
        exit(138);
    }
    let path = &args[2];

    match client.files_api.get_local_resource(path) {
        Ok(resources_result) => {
            print!("{resources_result}");
        }
        Err(e) => {
            eprint!("{}{}", e.http_code.unwrap_or(500), e.message);
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
    if args.len() < 5 {
        eprintln!("Usage: cli remote-before-copy <host> <port> <items>");
        exit(140);
    }
    let items = &args[4];
    let exit_code = client.remote_before_copy(items);

    if exit_code != 0 {
        exit(exit_code);
    }
}

pub fn command_local_before_copy(client: Client<'_>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 3 {
        eprintln!("Usage: cli local-before-copy <items>");
        exit(142);
    }

    let items = String::from(&args[2]);

    match client.files_api.local_before_copy(items) {
        Ok(response) => {
            print!("{}", response.trim().to_string());
        }
        Err(e) => {
            print!("{}", e.message);
            exit(150);
        }
    }
}
