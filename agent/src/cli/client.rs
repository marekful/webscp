use ssh2::Session;
use std::{
    fs,
    fs::OpenOptions,
    io::{prelude::*, Error},
    net::TcpStream,
    path::Path,
    process::exit,
    time::Instant,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

use std::process::Stdio;

use crate::{
    command_runner::{run_command, run_command_async},
    constants::{
        COMMAND_GET_LOCAL_RESOURCE, COMMAND_GET_LOCAL_USER, COMMAND_GET_LOCAL_VERSION,
        COMMAND_LOCAL_BEFORE_COPY, DEFAULTS,
    },
    files_api::FilesApi,
};

#[derive(Debug)]
pub struct Client<'r> {
    host: &'r str,
    port: i16,
    pub files_api: FilesApi,
}

pub struct ClientError {
    pub code: i32,
    pub message: String,
    pub http_code: Option<i32>,
}

impl From<std::io::Error> for ClientError {
    fn from(err: Error) -> Self {
        return ClientError {
            code: 987,
            message: err.to_string(),
            http_code: Some(500),
        };
    }
}

impl Client<'_> {
    pub fn new(host: &str, port: i16) -> Client {
        let files_api = FilesApi::new();
        Client {
            host,
            port,
            files_api,
        }
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

    pub fn get_remote_user(&self, user_name: &str, password: &str) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::remote_get_user(&sess, user_name, password) {
            Ok(resources_result) => {
                print!("{resources_result}");
            }
            Err(e) => {
                Client::print_error_and_exit(e.code, e.message);
            }
        }

        0
    }

    pub fn get_remote_resource(&self, user_id: u32, path: &str) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::remote_get_resource(&sess, user_id, &path) {
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

    pub fn remote_before_copy(&self, user_id: u32, items: &String) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::_remote_before_copy(&sess, user_id, &items) {
            Ok(result) => print!("{}", result),
            Err(e) => {
                eprint!("{}", e.message);
                return e.code;
            }
        }

        0
    }

    /*pub fn remote_do_copy(&self, archive_name: &String) -> i32 {
        /*let sess = self.create_session(None).unwrap();*/

        if let Err(e) = self._remote_do_copy(/*sess, */ &archive_name) {
            eprint!("{}{}", get_http_code_from_error(&e), e.message);
            return e.code;
        }

        0
    }*/

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

    pub async fn remote_do_copy_async(
        files_api: &FilesApi,
        host: &str,
        port: &str,
        archive_name: &str,
        remote_base_path: &str,
    ) -> Result<(), ClientError> {
        let local_path = format!(
            "{}{}{}",
            DEFAULTS.temp_data_dir, archive_name, ".agent.tar.gz"
        );
        let remote_path = format!("{}{}{}", DEFAULTS.temp_data_dir, archive_name, ".dst.tar");

        // create argument list for uploader script
        let mut script_args: Vec<&str> = Vec::new();
        script_args.push(DEFAULTS.uploader_script_path);
        script_args.push(&local_path);
        script_args.push(host);
        script_args.push(port);
        script_args.push(&remote_path);

        // setup command for asynchronous execution
        let mut cmd = Command::new("bash");
        cmd.args(script_args);
        cmd.stdout(Stdio::piped());
        let mut child = cmd.spawn().expect("spawn failed");
        let stdout = child.stdout.take().expect("stdout failed");

        // attach reader to command's stdout
        let mut reader = BufReader::new(stdout).lines();

        // kick off execution
        let archive_name_copy = String::from(archive_name);
        let upload_result = tokio::spawn(async move {
            let status = child.wait().await.expect("child error");

            let code = match status.code() {
                // catch SIGUSR1 here
                None => {
                    return Err(ClientError {
                        code: 346,
                        message: "aborted".to_string(),
                        http_code: Some(200),
                    })
                }
                Some(c) => c,
            };
            if code != 0 {
                let mut error = String::new();
                child
                    .stderr
                    .unwrap()
                    .read_to_string(&mut error)
                    .await
                    .unwrap();

                let err_msg = error.as_str();
                FilesApi::new()
                    .send_upload_status_update_async(&archive_name_copy, err_msg)
                    .await;

                return Err(ClientError {
                    code: 347,
                    message: error,
                    http_code: Some(500),
                });
            }

            Ok(())
        });

        files_api
            .send_upload_status_update_async(&archive_name, "starting upload")
            .await;

        // read lines from script output as they are written to its stdout
        while let Some(line) = reader.next_line().await? {
            let message = format!("progress::{}", line);
            files_api
                .send_upload_status_update_async(&archive_name, &message)
                .await;
        }

        // remove local copy of archive
        let mut rm_args: Vec<&str> = Vec::new();
        rm_args.push("-f");
        rm_args.push(local_path.as_str());
        let _rm_result = run_command_async(81, false, true, "rm", rm_args).await;

        // abort process on any errors from command execution (including usr1 signal)
        match upload_result.await.unwrap() {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        files_api
            .send_upload_status_update_async(&archive_name, "extracting")
            .await;

        // extract uploaded archive on remote
        let prt: i16 = port.to_string().parse().unwrap();
        let client = Client::new(host, prt);
        match client.remote_extract_archive(&archive_name, &remote_base_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                let err_msg = e.message.as_str();
                files_api
                    .send_upload_status_update_async(&archive_name, err_msg)
                    .await;
                Err(e)
            }
        }
    }

    pub fn remote_extract_archive(
        &self,
        archive_name: &str,
        remote_path: &str,
    ) -> Result<(), ClientError> {
        let sess = self.create_session(None).unwrap();
        let mut ch = sess.channel_session().unwrap();
        let archive_path = format!("{}{}.dst.tar", DEFAULTS.temp_data_dir, archive_name);
        let command = &*format!(
            "tar -xf {} -C {} && rm -rf {}",
            archive_path, remote_path, archive_path,
        );
        ch.exec(command).unwrap();
        let mut output = String::new();
        ch.read_to_string(&mut output).unwrap();

        let exit_code = ch.exit_status().unwrap();
        if exit_code != 0 {
            let mut stderr = String::new();
            ch.stderr().read_to_string(&mut stderr).unwrap();
            return Err(ClientError {
                code: exit_code,
                message: stderr,
                http_code: Some(503),
            });
        }

        Ok(())
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

    fn remote_get_resource(
        sess: &Session,
        user_id: u32,
        path: &str,
    ) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!(
            "{} {user_id} {path}",
            Client::command(COMMAND_GET_LOCAL_RESOURCE)
        );
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

    fn remote_get_user(
        sess: &Session,
        user_name: &str,
        password: &str,
    ) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!(
            "{} {user_name} {password}",
            Client::command(COMMAND_GET_LOCAL_USER)
        );
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

    fn _remote_before_copy(
        sess: &Session,
        user_id: u32,
        items: &str,
    ) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        ch.exec(&*format!(
            "{} {} {}",
            Client::command(COMMAND_LOCAL_BEFORE_COPY),
            user_id,
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

    /*fn _remote_do_copy(&self, archive_name: &str) -> Result<(), ClientError> {
        // create argument list for scp command
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

        // execute scp
        if let Err(e) = run_command(81, false, false, "scp", scp_args) {
            return Err(ClientError {
                code: e.code,
                message: e.message,
                http_code: Some(e.status.code as i32),
            });
        };

        send_upload_status_update(&archive_name, "extracting");

        // remove archive
        let mut rm_args: Vec<&str> = Vec::new();
        rm_args.push("-f");
        rm_args.push(local_path.as_str());

        let _rm_result = run_command(81, false, true, "rm", rm_args);

        // extract uploaded archive
        match self.remote_extract_archive(archive_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }*/

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
                eprintln!("Couldn't retrieve host key: ({}) {}", e.code, e.message);
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
