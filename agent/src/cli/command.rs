use std::process::exit;

use urlencoding::encode;

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

pub fn command_get_remote_resource(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 7 {
        eprintln!(
            "Usage: cli {} <host> <port> <user_id> <remote_token> <path>",
            COMMAND_GET_REMOTE_RESOURCE
        );
        exit(137);
    }
    let user_id: u32 = args[4].parse().unwrap_or(0);
    let remote_token = &args[5];
    let path = &args[6];

    client.get_remote_resource(user_id, remote_token, path);
}

pub fn command_get_local_resource(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!(
            "Usage: cli {} <user_id> <token> <path>",
            COMMAND_GET_LOCAL_RESOURCE
        );
        exit(138);
    }
    let user_id: u32 = args[2].parse().unwrap_or(0);
    let token = &args[3];
    let path = &args[4];
    let path_encoded = encode(path);

    match client
        .files_api
        .get_local_resource(user_id, token, &path_encoded)
    {
        Ok(resources_result) => {
            print!("{resources_result}");
        }
        Err(e) => {
            let mut msg = e.message;
            let http_code = match e.http_code {
                None => String::new(),
                Some(code) => {
                    msg = msg.replacen(&format!("{} ", code), "", 1);
                    format!("{} ", code)
                }
            };
            eprint!("{}{}", http_code, msg);
            exit(e.code);
        }
    }
}

pub fn command_get_remote_user(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 6 {
        eprintln!(
            "Usage: cli {} <host> <port> <username> <password>",
            COMMAND_GET_REMOTE_USER
        );
        exit(140);
    }
    let user_name = &args[4];
    let password = &args[5];

    client.get_remote_user(user_name, password);
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

pub fn command_get_token_user(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!(
            "Usage: cli {} <host> <port> <access_token>",
            COMMAND_GET_TOKEN_USER
        );
        exit(149);
    }
    let access_token = &args[4];

    let exit_code = client.get_token_user(access_token);

    if exit_code != 0 {
        exit(exit_code);
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
    let files_version = client.files_api.get_version();

    println!(
        "{{\"agent\": \"{}\", \"files\": \"{}\"}}",
        agent_version, files_version
    );
}

pub fn command_remote_before_copy(client: Client, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 7 {
        eprintln!(
            "Usage: cli {} <host> <port> <user_id> <remote_token> <items>",
            COMMAND_REMOTE_BEFORE_COPY
        );
        exit(140);
    }
    let user_id: u32 = args[4].parse().unwrap_or(0);
    let remote_token = &args[5];
    let items = &args[6];

    client.remote_before_copy(user_id, remote_token, items);
}

pub fn command_local_before_copy(client: Client<'_>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!(
            "Usage: cli {} <user_id> <token> <items>",
            COMMAND_LOCAL_BEFORE_COPY
        );
        exit(142);
    }

    let user_id: u32 = args[2].parse().unwrap_or(0);
    let token = String::from(&args[3]);
    let items = String::from(&args[4]);

    match client.files_api.local_before_copy(user_id, token, items) {
        Ok(response) => {
            print!("{}", response.trim());
        }
        Err(e) => {
            let mut msg = e.message;
            let http_code = match e.http_code {
                None => String::new(),
                Some(code) => {
                    msg = msg.replacen(&format!("{} ", code), "", 1);
                    format!("{} ", code)
                }
            };
            eprint!("{}{}", http_code, msg);
            exit(e.code);
        }
    }
}
