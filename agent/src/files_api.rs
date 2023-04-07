use reqwest::{blocking::Response, Response as AsyncResponse, StatusCode};
use std::{env, io::Read, time::Duration};

use crate::client::ClientError;

use crate::constants::DEFAULTS;

pub struct RequestError {
    pub code: i32,
    pub message: String,
    pub http_code: Option<u16>,
}

#[derive(Debug)]
pub struct FilesApi {
    base_url: String,
}

impl FilesApi {
    pub fn new() -> Self {
        Self {
            base_url: Self::get_base_url(),
        }
    }

    pub async fn send_upload_status_update_async(&self, transfer_id: &str, message: &str) {
        let uri = format!("/api/sse/transfers/{transfer_id}/update/{message}");

        let _ = self.make_async_get_request(&uri).await;

        ()
    }

    pub fn send_upload_status_update(&self, transfer_id: &str, message: &str) {
        let uri = format!("/api/sse/transfers/{transfer_id}/update/{message}");

        let _ = self.make_get_request(&uri);

        ()
    }

    pub fn get_local_resource(&self, user_id: u32, path: &str) -> Result<String, ClientError> {
        let uri = format!("/api/agent/{user_id}/resources/{path}");

        let mut response = match self.make_get_request(&uri) {
            Ok(r) => r,
            Err(e) => {
                return Err(ClientError {
                    code: 187,
                    message: e.message,
                    http_code: Some(500),
                });
            }
        };

        let mut output = String::new();
        let result = response.read_to_string(&mut output);

        if result.is_err() {
            return Err(ClientError {
                code: 188,
                message: result.unwrap_err().to_string(),
                http_code: Some(500),
            });
        }

        if response.status() != StatusCode::OK {
            return Err(ClientError {
                code: 189,
                message: response.status().to_string(),
                http_code: Some(response.status().as_u16() as i32),
            });
        }

        match response.read_to_string(&mut output) {
            Ok(_) => Ok(output),
            Err(e) => Err(ClientError {
                code: 190,
                message: e.to_string(),
                http_code: Some(500),
            }),
        }
    }

    pub fn local_before_copy(&self, user_id: u32, items: String) -> Result<String, ClientError> {
        let uri = format!("/api/agent/{user_id}/copy?action=remote-copy");

        let mut response = match self.make_post_request(&uri, items) {
            Ok(r) => r,
            Err(e) => {
                return Err(ClientError {
                    code: 191,
                    message: e.message,
                    http_code: Some(500),
                })
            }
        };

        let mut output = String::new();
        let result = response.read_to_string(&mut output);

        if result.is_err() {
            return Err(ClientError {
                code: 192,
                message: result.unwrap_err().to_string(),
                http_code: Some(500),
            });
        }

        if response.status() != StatusCode::OK {
            return Err(ClientError {
                code: 193,
                message: response.status().to_string(),
                http_code: Some(response.status().as_u16() as i32),
            });
        }

        match response.read_to_string(&mut output) {
            Ok(_) => Ok(output),
            Err(e) => Err(ClientError {
                code: 194,
                message: e.to_string(),
                http_code: Some(500),
            }),
        }
    }

    pub fn get_local_user(&self, user_name: &str, password: &str) -> Result<String, ClientError> {
        let uri = "/api/agent/verify-user-credentials";
        let request = format!(
            "{{\"name\": \"{}\", \"password\": \"{}\"}}",
            user_name, password
        );

        let mut response = match self.make_post_request(uri, request) {
            Ok(r) => r,
            Err(e) => {
                return Err(ClientError {
                    code: 195,
                    message: e.message,
                    http_code: Some(500),
                })
            }
        };

        let mut output = String::new();
        let result = response.read_to_string(&mut output);

        if result.is_err() {
            return Err(ClientError {
                code: 196,
                message: result.unwrap_err().to_string(),
                http_code: Some(500),
            });
        }

        if response.status() != StatusCode::OK {
            return Err(ClientError {
                code: 198,
                message: response.status().to_string(),
                http_code: Some(response.status().as_u16() as i32),
            });
        }

        match response.read_to_string(&mut output) {
            Ok(_) => Ok(output),
            Err(e) => Err(ClientError {
                code: 199,
                message: e.to_string(),
                http_code: Some(500),
            }),
        }
    }

    pub fn get_version(&self) -> String {
        let mut response = match self.make_get_request("/api/version") {
            Ok(r) => r,
            Err(_e) => return "unknown".to_string(),
        };

        let mut version = String::new();
        return match response.read_to_string(&mut version) {
            Ok(_) => version,
            Err(_) => "unknown".to_string(),
        };
    }

    fn make_post_request(&self, uri: &str, body: String) -> Result<Response, RequestError> {
        let request_url = format!("{}{uri}", self.base_url);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(100))
            .build()
            .unwrap();

        match client
            .post(request_url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
        {
            Ok(r) => Ok(r),
            Err(e) => Err(RequestError {
                code: 369,
                message: e.to_string(),
                http_code: Some(500),
            }),
        }
    }

    async fn make_async_get_request(&self, uri: &str) -> Result<AsyncResponse, RequestError> {
        let request_url = self.request_url(uri);

        return match reqwest::get(request_url).await {
            Ok(r) => Ok(r),
            Err(e) => {
                return Err(RequestError {
                    code: 370,
                    message: e.to_string(),
                    http_code: Some(500),
                });
            }
        };
    }

    fn make_get_request(&self, uri: &str) -> Result<Response, RequestError> {
        let request_url = self.request_url(uri);

        return match reqwest::blocking::get(request_url) {
            Ok(r) => Ok(r),
            Err(e) => {
                return Err(RequestError {
                    code: 371,
                    message: e.to_string(),
                    http_code: Some(500),
                });
            }
        };
    }

    fn request_url(&self, uri: &str) -> String {
        format!("{}{uri}", self.base_url)
    }

    fn get_base_url() -> String {
        let default_fb_api_address = DEFAULTS.default_fb_api_address;
        let fb_api_address_result = env::var(DEFAULTS.env_name_fb_api_address);
        return fb_api_address_result.unwrap_or(default_fb_api_address.to_string());
    }

    fn get_http_code_from_error(e: &ClientError) -> String {
        let http_code = e.http_code.unwrap_or(0);
        let mut http_code_str = String::new();
        if http_code > 0 {
            http_code_str = http_code.to_string() + " ";
        }

        http_code_str
    }
}
