mod archive;
mod client;
mod command;
mod command_runner;
mod constants;
#[path = "../fb_api_client.rs"]
mod fb_api;

use crate::{client::Client, command::*, constants::*};

use std::{env, process::exit};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut exec: Option<fn(Option<Client>, Option<Vec<String>>)> = None;
    //let mut exec2: dyn Future<Output=Result<fn(Option<Client>, Option<Vec<String>>), FutureError>> = None;

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
        COMMAND_LOCAL_BEFORE_COPY => exec = Some(command_local_before_copy),
        _ => {}
    }
    match exec {
        None => {} // proceed to remote commands
        Some(_) => {
            exec.unwrap()(None, Some(args));
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

    exec.unwrap()(Some(client), Some(args.clone()));
}
