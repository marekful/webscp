pub mod archive;
pub mod client;
pub mod command;
mod command_runner;
pub mod constants;
#[path = "../files_api.rs"]
mod files_api;

use crate::{client::Client, command::*, constants::*};

use std::{env, process::exit};

pub struct FutureCommandError {
    pub code: i32,
    pub message: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    #[allow(clippy::type_complexity)]
    let mut exec: Option<fn(Client, Option<Vec<String>>)> = None;

    // commands executed locally - these require no 'host' and 'port'
    // arguments but may require others
    if args.len() < 2 {
        eprintln!("Usage: cli <command> [arg, ..., argN]");
        exit(197);
    }
    let command = args[1].as_str();
    match command {
        COMMAND_GET_LOCAL_VERSION => exec = Some(command_get_local_version),
        COMMAND_GET_LOCAL_RESOURCE => exec = Some(command_get_local_resource),
        COMMAND_GET_LOCAL_USER => exec = Some(command_get_local_user),
        COMMAND_LOCAL_BEFORE_COPY => exec = Some(command_local_before_copy),
        _ => {}
    }
    match exec {
        None => {} // proceed to remote commands
        Some(_) => {
            let client = Client::new("", 0);
            exec.unwrap()(client, Some(args));
            exit(0);
        }
    }

    // commands executed on the remote agent require
    // 'host' and 'port' arguments as a minimum
    if args.len() < 4 {
        eprintln!("Usage: cli <command> <host> <port> [arg, ..., argN]");
        exit(128);
    }
    match command {
        COMMAND_EXCHANGE_KEYS => exec = Some(command_exchange_keys),
        COMMAND_GET_REMOTE_VERSION => exec = Some(command_get_remote_version),
        COMMAND_GET_REMOTE_RESOURCE => exec = Some(command_get_remote_resource),
        COMMAND_REMOTE_BEFORE_COPY => exec = Some(command_remote_before_copy),
        /*COMMAND_REMOTE_DO_COPY => exec = Some(command_remote_do_copy),*/
        COMMAND_GET_REMOTE_USER => exec = Some(command_get_remote_user),
        COMMAND_GET_TOKEN_USER => exec = Some(command_get_token_user),
        COMMAND_PING => exec = Some(command_ping),
        _ => {
            eprintln!("Invalid command {}", command);
            exit(129);
        }
    }

    let host = &args[2];
    let port_str = &args[3];
    let port = match port_str.parse::<i16>() {
        Ok(port) => port,
        Err(_e) => {
            eprintln!("Invalid port: {}", port_str);
            exit(130);
        }
    };
    let client = Client::new(host, port);

    exec.unwrap()(client, Some(args.clone()));
}
