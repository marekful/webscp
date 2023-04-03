use reqwest::StatusCode;
use std::{env, io::prelude::*, process::exit, time::Duration};

use crate::{client::*, constants::*};

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
        exit(137);
    }
    let path = &args[4];
    client.unwrap().get_remote_resource(path);
}

pub fn command_get_local_resource(_: Option<Client>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 3 {
        eprintln!("Usage: client {COMMAND_GET_LOCAL_RESOURCE} <path>");
        exit(138);
    }
    let path = &args[2];

    match get_local_resource_(path) {
        Ok(resources_result) => {
            print!("{resources_result}");
        }
        Err(e) => {
            eprint!("{}{}", get_http_code_from_error(&e), e.message);
            exit(e.code);
        }
    }
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

pub fn command_remote_before_copy(client: Option<Client>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: cli remote-before-copy <host> <port> <items>");
        exit(140);
    }
    let items = &args[4];
    let exit_code = client.unwrap().remote_before_copy(items);

    if exit_code != 0 {
        exit(exit_code);
    }
}

pub fn command_local_before_copy(_: Option<Client<'_>>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 3 {
        eprintln!("Usage: cli local-before-copy <items>");
        exit(142);
    }

    let items = String::from(&args[2]);
    let fb_api_address = get_fb_api_address();
    let request_url = format!("{}/api/agent/copy?action=remote-copy", fb_api_address);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(100))
        .build()
        .unwrap();

    let response = match client
        .post(request_url)
        .header("Content-Type", "application/json")
        .body(items)
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            print!("{}", e.to_string());
            exit(150);
        }
    };
    let code = response.status().as_u16() as i32;
    if response.status() != StatusCode::OK {
        print!("{}", response.status());
        exit(code);
    }

    print!("{}", response.text().unwrap().trim().to_string());
}

/*pub fn command_remote_do_copy(client: Option<Client<'_>>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: cli remote-do-copy <host> <port> <archive_name>");
        exit(143);
    }
    let archive_name = &args[4];

    let exit_code = client.unwrap().remote_do_copy(archive_name);

    if exit_code != 0 {
        exit(exit_code);
    }
}*/

fn get_fb_api_address() -> String {
    let default_fb_api_address = DEFAULTS.default_fb_api_address;
    let fb_api_address_result = env::var("FILEBROWSER_ADDRESS");
    return fb_api_address_result.unwrap_or(default_fb_api_address.to_string());
}

fn get_fb_version() -> String {
    let fb_api_address = get_fb_api_address();
    let mut response = match reqwest::blocking::get(fb_api_address + "/api/version") {
        Ok(r) => r,
        Err(_e) => return "unknown".to_string(),
    };

    let mut version = String::new();
    return match response.read_to_string(&mut version) {
        Ok(_) => version,
        Err(_) => "unknown".to_string(),
    };
}

pub fn get_local_resource_(path: &str) -> Result<String, ClientError> {
    let fb_api_address = get_fb_api_address();
    let request_url = fb_api_address + "/api/agent/resources/" + path;

    let mut response = match reqwest::blocking::get(request_url) {
        Ok(r) => r,
        Err(e) => {
            return Err(ClientError {
                code: 187,
                message: e.to_string(),
                http_code: Some(500),
            });
        }
    };

    let mut output = String::new();
    let result = response.read_to_string(&mut output);

    if result.is_err() {
        return Err(ClientError {
            code: 188,
            message: result.unwrap_err().to_string(),
            http_code: Some(500),
        });
    }

    if response.status() != StatusCode::OK {
        return Err(ClientError {
            code: 189,
            message: response.status().to_string(),
            http_code: Some(response.status().as_u16() as i32),
        });
    }

    return match response.read_to_string(&mut output) {
        Ok(_) => Ok(output),
        Err(e) => Err(ClientError {
            code: 190,
            message: e.to_string(),
            http_code: Some(500),
        }),
    };
}

pub fn get_http_code_from_error(e: &ClientError) -> String {
    let http_code = e.http_code.unwrap_or(0);
    let mut http_code_str = String::new();
    if http_code > 0 {
        http_code_str = http_code.to_string() + " ";
    }

    http_code_str
}
