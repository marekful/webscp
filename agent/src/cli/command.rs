use std::io::prelude::*;
use std::process::exit;
use reqwest::StatusCode;


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

fn get_fb_version() -> String {
    let fb_api_address = Client::get_fb_api_address();
    let mut response = match reqwest::blocking::get(fb_api_address + "/api/version") {
        Ok(r) => r,
        Err(_e) => return "unknown".to_string()
    };

    let mut version = String::new();
    return match response.read_to_string(&mut version) {
        Ok(_) => version,
        Err(_) => "unknown".to_string()
    }
}


fn get_local_resource(path: &str) {
    let fb_api_address = Client::get_fb_api_address();
    let request_url = fb_api_address + "/api/agent/resources/" + path;

    let mut response = match reqwest::blocking::get(request_url) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e.to_string());
            exit(187);
        }
    };

    let mut output = String::new();
    let result = response.read_to_string(& mut output);

    if response.status() != StatusCode::OK {
        println!("{}", output);
        exit(188);
    }

    if result.is_err() {
        println!("{}", result.unwrap_err().to_string());
        exit(189);
    }

    match response.read_to_string(&mut output) {
        Ok(_) => {
            println!("{}",  output);
        },
        Err(e) => {
            println!("{}", e.to_string());
            exit(190);
        }
    }
}
