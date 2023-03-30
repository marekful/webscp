use rocket::http::Status;
use std::process::{Command, Stdio};

use crate::constants::DEFAULTS;

pub struct CommandError {
    pub code: i32,
    pub message: String,
    pub status: Status,
}

pub fn run_command(
    command_id: i32,
    is_cli: bool,
    allow_stderr: bool,
    command: &str,
    args: Vec<&str>,
) -> Result<String, CommandError> {
    let mut command_args: Vec<&str> = Vec::new();
    let program;
    if is_cli == true {
        // prepend command to the provided list of arguments and execute cli as the program
        program = DEFAULTS.cli_executable_path;
        command_args.push(command);
        command_args.append(&mut args.clone());
    } else {
        // execute command as the program with the provided arguments
        program = command;
        command_args = args;
    }

    // attempt to execute command
    let result = Command::new(program)
        .args(command_args)
        .stdout(Stdio::piped())
        .output();

    // return error if failed to execute command
    if result.is_err() {
        return Err(CommandError {
            code: command_id,
            message: result.unwrap_err().to_string(),
            status: Status::InternalServerError,
        });
    }
    let result = result.unwrap();
    let stderr = String::from_utf8(result.stderr).unwrap_or("".to_string());
    let stdout = String::from_utf8(result.stdout).unwrap_or("".to_string());
    let code = result.status.code().unwrap();

    // return error if the command's error output is not empty
    if !allow_stderr && stderr.trim().len() > 0 {
        let s: String = stderr.chars().take(3).collect();
        let http_code: u16 = s.parse().unwrap_or(400);

        return Err(CommandError {
            code,
            message: stderr.trim().to_string() + " code:(" + code.to_string().as_str() + ")",
            status: Status::new(http_code),
        });
    }

    // return error if the command's return code is not zero
    if code != 0 {
        let s: String = stderr.chars().take(3).collect();
        let http_code: u16 = s.parse().unwrap_or(400);

        return Err(CommandError {
            code,
            message: stderr.trim().to_string() + " code:(" + code.to_string().as_str() + ")",
            status: Status::new(http_code),
        });
    }

    // return the commands standard output on success
    Ok(stdout)
}
