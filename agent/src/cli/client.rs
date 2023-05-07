use ssh2::Session;
use std::{
    fs,
    fs::OpenOptions,
    io::{prelude::*, Error},
    net::TcpStream,
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::exit,
    time::Instant,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

use std::process::Stdio;

use sha256::digest;

use crate::{
    command_runner::{run_command, run_command_async},
    constants::{
        COMMAND_GET_LOCAL_RESOURCE, COMMAND_GET_LOCAL_USER, COMMAND_GET_LOCAL_VERSION,
        COMMAND_LOCAL_BEFORE_COPY, DEFAULTS,
    },
    files_api::{FilesApi, Transfer},
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

impl From<Error> for ClientError {
    fn from(err: Error) -> Self {
        ClientError {
            code: 987,
            message: err.to_string(),
            http_code: Some(500),
        }
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

    pub fn print_error_and_exit(code: i32, message: String) {
        eprint!("{message}");
        exit(code);
    }

    pub fn random_hex() -> String {
        let key_id: u64 = rand::random::<u64>();
        let key_id_hex = format!("{:x}", key_id);

        key_id_hex
    }

    pub fn exchange_keys(&self, secret: &str) -> i32 {
        let sess = self.create_session(Some(secret)).unwrap();
        let send_result = self.send_public_key(&sess);
        if send_result != 0 {
            return send_result;
        }

        let receive_result = self.receive_host_key(&sess, secret);
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

    pub fn get_token_user(&self, secret: &str) -> i32 {
        let sess = self.create_session(Some(secret)).unwrap();
        match Client::remote_token_user(&sess, secret) {
            Ok(resources_result) => {
                print!("{resources_result}");
            }
            Err(e) => {
                Client::print_error_and_exit(e.code, e.message);
            }
        }

        0
    }

    pub fn get_remote_resource(&self, user_id: u32, token: &str, path: &str) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::remote_get_resource(&sess, user_id, token, path) {
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

    pub fn remote_before_copy(&self, user_id: u32, token: &str, items: &str) -> i32 {
        let sess = self.create_session(None).unwrap();
        match Client::_remote_before_copy(&sess, user_id, token, items) {
            Ok(result) => print!("{}", result),
            Err(e) => {
                Client::print_error_and_exit(e.code, e.message);
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

        println!(
            "{:.2}ms / {:.2}ms",
            dur_sess.as_millis(),
            dur_exec.as_millis()
        );

        0
    }

    pub async fn remote_do_copy_async(
        files_api: &FilesApi,
        transfer: &Transfer,
    ) -> Result<(), ClientError> {
        let archive_name = &transfer.transfer_id;
        let local_path = format!(
            "{}{}{}",
            DEFAULTS.temp_data_dir, archive_name, ".agent.tar.gz"
        );
        let remote_path = format!("{}{}{}", DEFAULTS.temp_data_dir, archive_name, ".dst.tar");

        // create argument list for uploader script
        let script_args: Vec<&str> = vec![
            DEFAULTS.uploader_script_path,
            &local_path,
            &transfer.host,
            &transfer.port,
            &remote_path,
        ];

        // setup command for asynchronous execution
        let mut cmd = Command::new("bash");
        cmd.args(script_args);
        cmd.stdout(Stdio::piped());
        let mut child = cmd.spawn().expect("spawn failed");
        let stdout = child.stdout.take().expect("stdout failed");

        files_api
            .send_upload_status_update_async(transfer, "uploading")
            .await;

        // attach reader to command's stdout
        let mut reader = BufReader::new(stdout).lines();

        // kick off execution
        let transfer_copy = transfer.clone();
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
                    .send_upload_status_update_async(&transfer_copy, err_msg)
                    .await;

                return Err(ClientError {
                    code: 347,
                    message: error,
                    http_code: Some(500),
                });
            }

            Ok(())
        });

        let msg = &format!("progress::stats::0/{}", transfer.size);
        files_api
            .send_upload_status_update_async(transfer, msg)
            .await;

        // read lines from script output as they are written to its stdout
        while let Some(line) = reader.next_line().await? {
            let message = format!("progress::{}", line);
            files_api
                .send_upload_status_update_async(transfer, &message)
                .await;
        }

        // remove local copy of archive
        let rm_args: Vec<&str> = vec!["-f", &local_path];
        let _rm_result = run_command_async(83, false, true, "rm", rm_args).await;

        // abort process on any errors from command execution (including usr1 signal)
        match upload_result.await.unwrap() {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        files_api
            .send_upload_status_update_async(transfer, "extracting")
            .await;

        // extract uploaded archive on remote
        let port: i16 = transfer.port.to_string().parse::<i16>().unwrap();
        let client = Client::new(&transfer.host, port);
        match client.remote_extract_archive(
            archive_name,
            &transfer.remote_path,
            transfer.compress,
            transfer.overwrite,
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                let err_msg = e.message.as_str();
                files_api
                    .send_upload_status_update_async(transfer, err_msg)
                    .await;
                Err(e)
            }
        }
    }

    pub fn remote_extract_archive(
        &self,
        archive_name: &str,
        remote_path: &str,
        is_compressed: bool,
        overwrite: bool,
    ) -> Result<(), ClientError> {
        let sess = self.create_session(None).unwrap();
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!(
            "{} \"{}\" \"{}\" {} {}",
            DEFAULTS.extract_archive_script_path,
            archive_name,
            remote_path,
            is_compressed,
            overwrite,
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

    fn create_session(&self, secret: Option<&str>) -> Option<Session> {
        // create ssh connection
        let tcp = match TcpStream::connect(self.host.to_owned() + ":" + &*self.port.to_string()) {
            Ok(tcp) => tcp,
            Err(e) => {
                Client::print_error_and_exit(
                    132,
                    format!("503 Couldn't connect to {}:{}: {}", self.host, self.port, e),
                );
                return None;
            }
        };
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        match secret {
            // authenticate session via default public-key
            None => {
                let pubkey: &Path = Path::new(DEFAULTS.public_key_file);
                let privkey: &Path = Path::new(DEFAULTS.private_key_file);
                match sess.userauth_pubkey_file("agent", Some(pubkey), privkey, None) {
                    Ok(r) => r,
                    Err(_e) => {
                        eprintln!("401 Public key authentication failed");
                        exit(135);
                    }
                }
            }
            // authenticate session via temporary private-key
            Some(secret) => {
                let key_id = Self::random_hex();
                let key_id_copy = key_id.clone();
                Self::create_key_file_from_access_token(key_id, secret.to_string());

                let path = format!("{}-{}-atmp", DEFAULTS.temporary_key_file_name, key_id_copy);
                let privkey: &Path = Path::new(&path);
                match sess.userauth_pubkey_file("agent", None, privkey, None) {
                    Ok(r) => r,
                    Err(_e) => {
                        eprintln!("401 Invalid access token");
                        exit(134);
                    }
                };

                Self::remove_key_file(key_id_copy.as_str());
            }
        }

        Some(sess)
    }

    fn send_public_key(&self, sess: &Session) -> i32 {
        // read our public key
        let key = &fs::read_to_string(DEFAULTS.public_key_file).unwrap();

        // upload our public key
        let mut upload = sess.channel_session().unwrap();
        upload
            .exec(&format!(
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

    fn receive_host_key(&self, sess: &Session, secret: &str) -> i32 {
        // download their public key
        /*let mut download = sess.channel_session().unwrap();
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
        }*/

        // retrieve their host key
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

        // add their host key
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(DEFAULTS.known_hosts_file)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", &host_key.trim()) {
            eprintln!("Couldn't write to file: {}", e);
        }

        // remove lock file on remote: this will signal that the temporary key
        // used for authenticating this key exchange session can be revoked
        let lock_file = digest(secret);
        let mut result = sess.channel_session().unwrap();
        result
            .exec(&format!("rm -f {}/{}", DEFAULTS.ssh_dir_path, lock_file))
            .unwrap();
        let mut key = String::new();
        result.read_to_string(&mut key).unwrap();

        0
    }

    fn create_key_file_from_access_token(key_id: String, secret: String) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o600)
            .open(format!(
                "{}-{}-atmp",
                DEFAULTS.temporary_key_file_name, key_id
            ))
            .unwrap();

        let secret_lines = secret
            .chars()
            .collect::<Vec<char>>()
            .chunks(64)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        if let Err(e) = writeln!(
            file,
            "-----BEGIN EC PRIVATE KEY-----\n{}\n-----END EC PRIVATE KEY-----",
            secret_lines
        ) {
            eprintln!("Couldn't write to file: {}", e);
            exit(133);
        }
    }

    fn remove_key_file(key_id: &str) {
        match fs::remove_file(format!(
            "{}-{}-atmp",
            DEFAULTS.temporary_key_file_name, key_id
        )) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Couldn't remove file: {}", e);
                exit(131);
            }
        }
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
        token: &str,
        path: &str,
    ) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!(
            "{} {user_id} {token} {path}",
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

    fn remote_token_user(sess: &Session, access_token: &str) -> Result<String, ClientError> {
        let token_hash = digest(access_token);
        let mut ch = sess.channel_session().unwrap();
        let command = &*format!("cat {}/{}", DEFAULTS.ssh_dir_path, token_hash);
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
        token: &str,
        items: &str,
    ) -> Result<String, ClientError> {
        let mut ch = sess.channel_session().unwrap();
        ch.exec(&format!(
            "{} {} {} {}",
            Client::command(COMMAND_LOCAL_BEFORE_COPY),
            user_id,
            token,
            items
        ))
        .unwrap();

        let mut output = String::new();
        ch.read_to_string(&mut output).unwrap();

        let exit_code = ch.exit_status().unwrap();
        if exit_code != 0 {
            let mut err_msg = String::new();
            let _ = ch.stderr().read_to_string(&mut err_msg);
            return Err(ClientError {
                code: exit_code,
                message: err_msg,
                http_code: None,
            });
        }

        Ok(output)
    }
}
