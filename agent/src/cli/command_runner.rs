use rocket::http::Status;
use std::process::{Command, Stdio};

use tokio::process::Command as AsyncCommand;

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
    let (program, command_args) = get_command_args(command, is_cli, args);

    // attempt to execute command
    let result = Command::new(program)
        .args(command_args)
        .stdout(Stdio::piped())
        .output();

    // return error if failed to execute command
    if let Err(err) = result {
        return Err(get_error(command_id, err.to_string(), "500"));
    }
    let result = result.unwrap();
    let stderr = String::from_utf8(result.stderr).unwrap_or("".to_string());
    let stdout = String::from_utf8(result.stdout).unwrap_or("".to_string());
    let code = result.status.code().unwrap();

    // return error if the command's error output is not empty
    if !allow_stderr && !stderr.trim().is_empty() {
        return Err(get_error(code, stderr.trim(), &stderr));
    }

    // return error if the command's return code is not zero
    if code != 0 {
        return Err(get_error(code, stderr.trim(), &stderr));
    }

    // return the commands standard output on success
    Ok(stdout)
}

pub async fn run_command_async(
    command_id: i32,
    is_cli: bool,
    allow_stderr: bool,
    command: &str,
    args: Vec<&str>,
) -> Result<String, CommandError> {
    let (program, command_args) = get_command_args(command, is_cli, args);

    // setup and execute command
    let mut cmd = AsyncCommand::new(program);
    let result = cmd.args(command_args).output().await;

    // return error if failed to execute command
    if let Err(err) = result {
        return Err(get_error(command_id, err.to_string(), "500"));
    }

    let result = result.unwrap();
    let stderr = String::from_utf8(result.stderr).unwrap_or("".to_string());
    let stdout = String::from_utf8(result.stdout).unwrap_or("".to_string());
    let code = result.status.code().unwrap();

    // return error if the command's error output is not empty
    if !allow_stderr && !stderr.trim().is_empty() {
        return Err(get_error(code, stderr.trim(), &stderr));
    }

    // return error if the command's return code is not zero
    if code != 0 {
        return Err(get_error(code, stderr.trim(), &stderr));
    }

    // return the commands standard output on success
    Ok(stdout)
}

fn get_command_args<'a>(
    command: &'a str,
    is_cli: bool,
    args: Vec<&'a str>,
) -> (&'a str, Vec<&'a str>) {
    let mut command_args: Vec<&str> = Vec::new();
    let program;
    if is_cli {
        // prepend command to the provided list of arguments and execute cli as the program
        program = DEFAULTS.cli_executable_path;
        command_args.push(command);
        command_args.append(&mut args.clone());
    } else {
        // execute command as the program with the provided arguments
        program = command;
        command_args = args;
    }

    (program, command_args)
}

fn get_error<TM: Into<String>, TE: Into<String>>(
    code: i32,
    message: TM,
    stderr: TE,
) -> CommandError {
    let s: String = stderr.into().chars().take(3).collect();
    let http_code: u16 = s.parse().unwrap_or(400);
    let msg = message.into().replacen(&format!("{} ", http_code), "", 1);

    CommandError {
        code,
        message: format!("{} (code:{})", msg, code),
        status: Status::new(http_code),
    }
}
