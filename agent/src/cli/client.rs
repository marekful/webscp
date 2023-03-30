use ssh2::Session;
use std::{
    fs, fs::OpenOptions, io::prelude::*, net::TcpStream, path::Path, process::exit, time::Instant,
};

use crate::{
    command::*,
    command_runner::run_command,
    constants::{
        COMMAND_GET_LOCAL_RESOURCE, COMMAND_GET_LOCAL_VERSION, COMMAND_LOCAL_BEFORE_COPY, DEFAULTS,
    },
    fb_api::send_upload_status_update,
};

#[derive(Debug)]
pub struct Client<'r> {
    host: &'r str,
    port: i16,
}

pub struct ClientError {
    pub code: i32,
    pub message: String,
    pub http_code: Option<i32>,
}

impl Client<'_> {
    pub fn new(host: &str, port: i16) -> Client {
        Client { host, port }
    }

    pub fn command(command: &str) -> String {
        format!(
            "{} {} {}",
            DEFAULTS.with_contenv, DEFAULTS.cli_executable_path, command
        )
    }

    pub fn exchange_keys(&self, secret: &str) -> i32 {
        let sess = self.create_session(Some(secret)).unwrap();
        let send_result = self.send_public_key(&sess);
        if send_result != 0 {
            return send_result;
        }

        let receive_result = self.receive_public_key(&sess);
        if receive_result != 0 {
            return receive_result;
        }

        0
    }

    pub fn get_remote_resource(&self, path: &str) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::_get_remote_resource(&sess, &path) {
            Ok(resources_result) => {
                print!("{resources_result}");
            }
            Err(e) => {
                Client::print_error_and_exit(e.code, e.message);
            }
        }

        0
    }

    pub fn get_remote_version(&self) -> i32 {
        let sess = self.create_session(None).unwrap();
        let version = Client::get_agent_version(&sess);

        print!("{version}");

        0
    }

    pub fn remote_before_copy(&self, items: &String) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::_remote_before_copy(&sess, &items) {
            Ok(result) => print!("{}", result),
            Err(e) => {
                eprint!("{}", e.message);
                return e.code;
            }
        }

        0
    }

    pub fn remote_do_copy(&self, archive_name: &String) -> i32 {
        let sess = self.create_session(None).unwrap();

        match self._remote_do_copy(&sess, &archive_name) {
            Ok(_) => {}
            Err(e) => {
                eprint!("{}{}", get_http_code_from_error(&e), e.message);
                return e.code;
            }
        }

        0
    }

    pub fn ping(&self) -> i32 {
        // start duration measure
        let start = Instant::now();
        let sess = self.create_session(None).unwrap();
        // authenticated session duration
        let dur_sess = start.elapsed();
        let mut ch = sess.channel_session().unwrap();
        ch.exec("(exec :)").unwrap();
        // exec duration
        let dur_exec = start.elapsed();

        let result = ch.exit_status().unwrap();
        if result != 0 {
            return result;
        }

        print!("{:?} {:?}\n", dur_sess, dur_exec);

        0
    }

    pub fn print_error_and_exit(code: i32, message: String) {
        eprint!("{message}");
        exit(code);
    }

    fn create_session(&self, secret: Option<&str>) -> Option<Session> {
        // create ssh connection
        let tcp = match TcpStream::connect(self.host.to_owned() + ":" + &*self.port.to_string()) {
            Ok(tcp) => tcp,
            Err(e) => {
                Client::print_error_and_exit(
                    132,
                    format!(
                        "503 Couldn't connect to {}:{}: {}",
                        self.host,
                        self.port,
                        e.to_string()
                    ),
                );
                return None;
            }
        };
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        match secret {
            // authenticate session via public key
            None => {
                let pubkey: &Path = Path::new(DEFAULTS.public_key_file);
                let privkey: &Path = Path::new(DEFAULTS.private_key_file);
                match sess.userauth_pubkey_file("agent", Some(pubkey), privkey, None) {
                    Ok(r) => r,
                    Err(_e) => {
                        eprintln!("Public key authentication failed");
                        exit(135);
                    }
                }
            }
            // authenticate session via password (for exchange-keys)
            Some(s) => {
                match sess.userauth_password("agent", s) {
                    Ok(r) => r,
                    Err(_e) => {
                        eprintln!("Invalid agent secret");
                        exit(134);
                    }
                };
            }
        }

        Some(sess)
    }

    fn get_agent_version(sess: &Session) -> String {
        let mut ch = sess.channel_session().unwrap();
        ch.exec(Client::command(COMMAND_GET_LOCAL_VERSION).as_str())
            .unwrap();
        let mut version = String::new();
        ch.read_to_string(&mut version).unwrap();

        let result = ch.exit_status().unwrap();
        if result != 0 {
            return "".to_string();
        }

        version
    }

    fn _get_remote_resource(sess: &Session, path: &str) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!("{} {path}", Client::command(COMMAND_GET_LOCAL_RESOURCE));
        ch.exec(command).unwrap();
        let mut output = String::new();
        let mut stderr = String::new();
        ch.read_to_string(&mut output).unwrap();
        ch.stderr().read_to_string(&mut stderr).unwrap();

        let result = ch.exit_status().unwrap();

        if result == 0 {
            return Ok(output);
        }

        Err(ClientError {
            message: stderr,
            code: result,
            http_code: None,
        })
    }

    fn _remote_before_copy(sess: &Session, items: &str) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        ch.exec(&*format!(
            "{} {}",
            Client::command(COMMAND_LOCAL_BEFORE_COPY),
            items
        ))
        .unwrap();
        let mut output = String::new();
        ch.read_to_string(&mut output).unwrap();

        let exit_code = ch.exit_status().unwrap();
        if exit_code != 0 {
            return Err(ClientError {
                code: exit_code,
                message: output,
                http_code: None,
            });
        }

        Ok(output)
    }

    fn _remote_extract_archive(sess: &Session, archive_name: &str) -> Result<(), ClientError> {
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!(
            "{} {}{}{}",
            "tar --ignore-failed-read -xf",
            DEFAULTS.temp_data_dir,
            archive_name,
            ".dst.tar -C /srv"
        );
        ch.exec(command).unwrap();
        let mut output = String::new();
        ch.read_to_string(&mut output).unwrap();

        let exit_code = ch.exit_status().unwrap();
        if exit_code != 0 {
            return Err(ClientError {
                code: exit_code,
                message: command.to_string(),
                http_code: Some(503),
            });
        }

        Ok(())
    }

    fn _remote_do_copy(&self, sess: &Session, archive_name: &str) -> Result<(), ClientError> {
        let local_path = format!(
            "{}{}{}",
            DEFAULTS.temp_data_dir, archive_name, ".agent.tar.gz"
        );
        let remote_path = format!("{}{}{}", DEFAULTS.temp_data_dir, archive_name, ".dst.tar");

        let remote_scp_path = format!("{}:{}", self.host, remote_path);
        let port = self.port.to_string();
        let mut scp_args: Vec<&str> = Vec::new();

        scp_args.push("-P");
        scp_args.push(port.as_str());
        scp_args.push(local_path.as_str());
        scp_args.push(remote_scp_path.as_str());

        match run_command(81, false, false, "scp", scp_args) {
            Ok(_) => {}
            Err(err) => {
                return Err(ClientError {
                    code: err.code,
                    message: err.message,
                    http_code: Some(err.status.code as i32),
                })
            }
        };

        send_upload_status_update(&archive_name, "extracting");

        // extract uploaded archive
        match Client::_remote_extract_archive(sess, archive_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn send_public_key(&self, sess: &Session) -> i32 {
        // read our public key
        let key = &fs::read_to_string(DEFAULTS.public_key_file).unwrap();

        // upload our public key
        let mut upload = sess.channel_session().unwrap();
        upload
            .exec(&*format!(
                "echo -n \"{key}\" >> {}",
                DEFAULTS.authorized_keys_file
            ))
            .unwrap();

        // exit if we couldn't upload the key
        let upload_result = upload.exit_status().unwrap();
        if upload_result != 0 {
            return upload_result;
        }

        0
    }

    fn receive_public_key(&self, sess: &Session) -> i32 {
        // download their public key
        let mut download = sess.channel_session().unwrap();
        download
            .exec(&*format!("cat {}", DEFAULTS.public_key_file))
            .unwrap();
        let mut key = String::new();
        download.read_to_string(&mut key).unwrap();

        // exit if we couldn't download the key
        let download_result = download.exit_status().unwrap();
        if download_result != 0 {
            return download_result;
        }

        // add their public key
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(DEFAULTS.authorized_keys_file)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", &key.trim()) {
            eprintln!("Couldn't write to file: {}", e);
        }

        // add their host key
        let mut args: Vec<&str> = Vec::new();
        let port_str = self.port.to_string();
        args.push("-H");
        args.push("-p");
        args.push(&port_str);
        args.push("-t");
        args.push("ecdsa");
        args.push(self.host);
        let host_key = match run_command(318, false, true, "ssh-keyscan", args) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Couldn't retrieve host key: ({}) {}",e.code, e.message);
                return e.code;
            }
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(DEFAULTS.known_hosts_file)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", &host_key.trim()) {
            eprintln!("Couldn't write to file: {}", e);
        }

        0
    }
}
