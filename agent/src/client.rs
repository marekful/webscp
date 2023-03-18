use ssh2::{Session};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use std::time::Instant;

pub struct Client {
    host: String,
    port: i16,
}

impl Client {
    pub fn new(host: String, port: i16) -> Client {
        Client { host, port }
    }

    pub fn exchange_keys(&self, secret: &str) -> i32 {
        let sess = self.create_session(Some(secret));

        let send_result = Client::send_public_key(&sess);
        if send_result != 0 {
            return send_result;
        }

        let receive_result = Client::receive_public_key(&sess);
        if receive_result != 0 {
            return receive_result;
        }

        0
    }

    pub fn get_version(&self) -> i32 {
        let sess = self.create_session(None);
        let version = self.get_agent_version(&sess);

        print!("{}", version);

        0
    }

    pub fn ping(&self) -> i32 {

        // start duration measure
        let start = Instant::now();
        let sess = self.create_session(None);
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

    fn create_session(&self, secret: Option<&str>) -> Session {
        // Create SSH connection
        let tcp = match TcpStream::connect(self.host.to_owned() + ":" + &*self.port.to_string()) {
            Ok(tcp) => tcp,
            Err(_e) => {
                eprintln!("Couldn't connect to {}:{}", self.host, self.port);
                exit(132);
            }
        };
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        match secret {
            // Authenticate with public key
            None => {
                let pubkey: &Path = Path::new("/home/agent/.ssh/id_rsa.pub");
                let privkey: &Path = Path::new("/home/agent/.ssh/id_rsa");
                match sess.userauth_pubkey_file("agent", Some(pubkey), privkey, None) {
                    Ok(r) => r,
                    Err(_e) => {
                        eprintln!("Public key authentication failed");
                        exit(135);
                    }
                }
            }
            // Authenticate with agent secret
            Some(_secret) => {
                match sess.userauth_password("agent", _secret) {
                    Ok(r) => r,
                    Err(_e) => {
                        eprintln!("Invalid agent secret");
                        exit(134);
                    }
                };
            }
        }

        sess
    }

    fn get_agent_version(&self, sess: &Session) -> String {
        let mut ch = sess.channel_session().unwrap();
        ch.exec("with-contenv /app/target/debug/client version").unwrap();
        let mut version = String::new();
        ch.read_to_string(&mut version).unwrap();

        let result = ch.exit_status().unwrap();
        if result != 0 {
            return "".to_string();
        }

        return version
    }

    fn send_public_key(sess: &Session) -> i32 {
        // Read our public key
        let key = &fs::read_to_string("/home/agent/.ssh/id_rsa.pub").unwrap();

        // Upload our public key
        let mut upload = sess.channel_session().unwrap();
        upload
            .exec(&*format!(
                "echo -n \"{key}\" >> /home/agent/.ssh/authorized_keys"
            ))
            .unwrap();

        // Exit if we couldn't upload the key
        let upload_result = upload.exit_status().unwrap();
        if upload_result != 0 {
            return upload_result;
        }

        0
    }

    fn receive_public_key(sess: &Session) -> i32 {
        // Download their public key
        let mut download = sess.channel_session().unwrap();
        download.exec("cat /home/agent/.ssh/id_rsa.pub").unwrap();
        let mut key = String::new();
        download.read_to_string(&mut key).unwrap();

        // Exit if we couldn't download the key
        let download_result = download.exit_status().unwrap();
        if download_result != 0 {
            return download_result;
        }

        // Add their key
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("/home/agent/.ssh/authorized_keys")
            .unwrap();

        if let Err(e) = writeln!(file, "{}", &key.trim()) {
            eprintln!("Couldn't write to file: {}", e);
        }

        0
    }
}

fn get_fb_version() -> String {
    let default_fb_api_address = "http://filebrowser:7000".to_string();
    let fb_api_address_result = env::var("FILEBROWSER_ADDRESS");
    let fb_api_address = fb_api_address_result.unwrap_or(default_fb_api_address);

    let mut response = match reqwest::blocking::get(fb_api_address + "/api/version") {
        Ok(r) => r,
        Err(_e) => return "unknown".to_string()
    };

    let mut version = String::new();
    return match response.read_to_string(&mut version) {
        Ok(_) => version,
        Err(_) => "unknown".to_string()
    }
}

fn command_exchange_keys(client: Option<Client>, args: Option<Vec<String>>) {
    let args = args.unwrap();
    if args.len() < 5 {
        eprintln!("Usage: client exchange-keys <host> <port> <agent_secret>");
        exit(136);
    }
    let secret = &args[4];
    client.unwrap().exchange_keys(secret);
}

fn command_remote_version(client: Option<Client>, _: Option<Vec<String>>) {
    client.unwrap().get_version();
}

fn command_ping(client: Option<Client>, _: Option<Vec<String>>) {
    client.unwrap().ping();
}

fn command_version(_: Option<Client>, _: Option<Vec<String>>) {
    const AGENT_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    let agent_version = AGENT_VERSION.unwrap_or("unknown").to_string();
    let fb_version = get_fb_version();

    println!("{} / {}", agent_version, fb_version);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut exec: Option<fn(Option<Client>, Option<Vec<String>>)> = None;

    // The 'version' command is executed locally and requires no arguments
    if args.len() < 2 {
        eprintln!("Usage: client <command> [<host> <port> [arg, ..., argN]]");
        exit(127);
    }
    let command = args[1].as_str();
    match command {
        "version" => exec = Some(command_version),
        _ => {}
    }
    match exec {
        None => {}
        Some(_) => {
            exec.unwrap()(None, None);
            exit(0);
        }
    }

    // All other commands require 'host' and 'port' as a minimum
    if args.len() < 4 {
        eprintln!("Usage: client <command> <host> <port> [arg, ..., argN]");
        exit(127);
    }
    match command {
        "exchange-keys" => {
            exec = Some(command_exchange_keys);
        }
        "ping" => {
            exec = Some(command_ping);
        }
        "remote-version" => {
            exec = Some(command_remote_version);
        }
        _ => {
            eprintln!("Invalid command {}", command);
            exit(129);
        }
    }

    let host = args[2].to_string();
    let port_str = &args[3];
    let port = match port_str.parse::<i16>() {
        Ok(port) => port,
        Err(_e) => {
            eprintln!("Invalid port: {}", port_str);
            exit(128);
        }
    };
    let client = Client::new(host, port);

    exec.unwrap()(Some(client), Some(args));
}
