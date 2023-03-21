use ssh2::{Session};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use std::time::Instant;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct Client<'r> {
    host: &'r str,
    port: i16,
    default_fb_api_address: &'r str,
}

impl Default for Client<'_> {
    fn default() -> Self {
        Self {
            host: "",
            port: 0,
            default_fb_api_address: "http://filebrowser:80",
        }
    }
}

impl Client<'_> {
    pub fn new(host: &str, port: i16) -> Client {
        Client { host, port, ..Default::default() }
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

    pub fn get_remote_resource(&self, path: &str) -> i32 {
        let sess = self.create_session(None);
        let resources_result = Client::_get_remote_resource(&sess, &path);

        print!("{}", resources_result);

        0
    }

    pub fn get_remote_version(&self) -> i32 {
        let sess = self.create_session(None);
        let version = Client::get_agent_version(&sess);

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
        // create ssh connection
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
            // authenticate session via password (for exchange-keys)
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
            // authenticate session via public key
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

    pub fn get_fb_api_address() -> String {
        let default_fb_api_address = Client::default().default_fb_api_address;
        let fb_api_address_result = env::var("FILEBROWSER_ADDRESS");
        return fb_api_address_result.unwrap_or(default_fb_api_address.to_string());
    }

    fn get_agent_version(sess: &Session) -> String {
        let mut ch = sess.channel_session().unwrap();
        ch.exec("with-contenv /app/target/debug/cli get-local-version").unwrap();
        let mut version = String::new();
        ch.read_to_string(&mut version).unwrap();

        let result = ch.exit_status().unwrap();
        if result != 0 {
            return "".to_string();
        }

        version
    }

    fn _get_remote_resource(sess: &Session, path: &str) -> String {
        let mut ch = sess.channel_session().unwrap();
        ch.exec(&*format!("with-contenv /app/target/debug/cli get-local-resource {}", path)).unwrap();
        let mut output = String::new();
        ch.read_to_string(&mut output).unwrap();

        let result = ch.exit_status().unwrap();
        if result != 0 {
            eprint!("{{\"code\": {}, \"error\": \"{}\"}}", result, output.trim());
            return "".to_string();
        }

        return output;
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

pub fn get_fb_version() -> String {
    let fb_api_address = Client::get_fb_api_address();
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

pub fn get_local_resource(path: &str) {
    let fb_api_address = Client::get_fb_api_address();
    let request_url = fb_api_address + "/api/agent/resources/" + path;

    let mut response = match reqwest::blocking::get(request_url) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e.to_string());
            exit(187);
        }
    };

    let mut output = String::new();
    let result = response.read_to_string(& mut output);

    if response.status() != StatusCode::OK {
        println!("{}", output);
        exit(188);
    }

    if result.is_err() {
        println!("{}", result.unwrap_err().to_string());
        exit(189);
    }

    match response.read_to_string(&mut output) {
        Ok(_) => {
            println!("{}",  output);
        },
        Err(e) => {
            println!("{}", e.to_string());
            exit(190);
        }
    }
}
