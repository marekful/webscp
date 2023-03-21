mod client;
mod command;

use crate::client::Client;
use crate::command::*;

use std::env;
use std::process::exit;


fn main() {

    let args: Vec<String> = env::args().collect();
    let mut exec: Option<fn(Option<Client>, Option<Vec<String>>)> = None;

    // commands executed locally - these requires no arguments
    if args.len() < 2 {
        eprintln!("Usage: client <command> [<host> <port> [arg, ..., argN]]");
        exit(127);
    }
    let command = args[1].as_str();
    match command {
        "get-local-version"     => exec = Some(command_get_local_version),
        "get-local-resource"   => exec = Some(command_get_local_resource),
        _ => {}
    }
    match exec {
        None => {}
        Some(_) => {
            exec.unwrap()(None, Some(args));
            exit(0);
        }
    }

    // commands executed on the remote agent require
    // 'host' and 'port' arguments as a minimum
    if args.len() < 4 {
        eprintln!("Usage: client <command> <host> <port> [arg, ..., argN]");
        exit(127);
    }
    match command {
        "exchange-keys"         => exec = Some(command_exchange_keys),
        "get-remote-version"    => exec = Some(command_get_remote_version),
        "get-remote-resource"   => exec = Some(command_get_remote_resource),
        "ping"                  => exec = Some(command_ping),
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
            exit(128);
        }
    };
    let client = Client::new(host, port);

    exec.unwrap()(Some(client), Some(args.clone()));
}
