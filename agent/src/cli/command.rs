use std::process::exit;

use crate::client::*;

pub fn command_exchange_keys(client: Option<Client>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: client exchange-keys <host> <port> <agent_secret>");
        exit(136);
    }
    let secret = &args[4];
    client.unwrap().exchange_keys(secret);
}

pub fn command_get_remote_resource(client: Option<Client>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: client get-remote-resource <host> <port> <path>");
        exit(136);
    }
    let path = &args[4];
    client.unwrap().get_remote_resource(path);
}

pub fn command_get_local_resource(_: Option<Client>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 3 {
        eprintln!("Usage: client get-local-resource <path>");
        exit(136);
    }
    let path = &args[2];
    get_local_resource(path);

}

pub fn command_get_remote_version(client: Option<Client>, _: Option<Vec<String>>) {
    client.unwrap().get_remote_version();
}

pub fn command_ping(client: Option<Client>, _: Option<Vec<String>>) {
    client.unwrap().ping();
}

pub fn command_get_local_version(_: Option<Client>, _: Option<Vec<String>>) {
    const AGENT_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    let agent_version = AGENT_VERSION.unwrap_or("unknown").to_string();
    let fb_version = get_fb_version();

    println!("{} / {}", agent_version, fb_version);
}