use reqwest::{blocking::Response, Response as AsyncResponse, StatusCode};
use rocket::{http::Cookie, serde::json::serde_json};
use std::{env, io::Read, time::Duration};

use serde::Deserialize;

use crate::client::ClientError;

use crate::constants::DEFAULTS;

#[derive(Deserialize, Debug)]
pub struct Agent {
    pub id: u32,
    #[serde(alias = "userID")]
    pub user_id: u32,
    pub host: String,
    pub port: String,
    pub remote_user: RemoteUser,
}

#[derive(Deserialize, Debug)]
pub struct RemoteUser {
    pub id: u32,
    pub name: String,
    pub root: String,
}

#[derive(Debug)]
pub struct RequestError {
    pub code: i32,
    pub message: String,
    pub http_code: Option<u16>,
}

#[derive(Debug)]
pub struct FilesApi {
    base_url: String,
}

pub struct Transfer {
    pub agent_id: u32,
    pub transfer_id: String,
    pub rc_auth: String,
}

impl Clone for Transfer {
    fn clone(&self) -> Self {
        Self {
            agent_id: self.agent_id.clone(),
            transfer_id: self.transfer_id.clone(),
            rc_auth: self.rc_auth.clone(),
        }
    }
}

impl FilesApi {
    pub fn new() -> Self {
        Self {
            base_url: Self::get_base_url(),
        }
    }

    /// Makes an authenticated request back to Files API using the user's
    /// current JWT token to fetch the referred Agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The referred Agent ID
    /// * `auth_cookie` - The result of `CookieJar::get("rc_auth")` on the
    ///    incoming API request. If not `None`, a JWT token valid in Files backend
    pub async fn get_agent(
        &self,
        agent_id: u32,
        auth_cookie: Option<&Cookie<'_>>,
    ) -> Result<(Agent, String), RequestError> {
        // fail if cannot unwrap cookie value
        if auth_cookie.is_none() {
            return Err(RequestError {
                code: 414,
                message: "".to_string(),
                http_code: Some(401),
            });
        }
        let auth_token = auth_cookie.unwrap().value();

        // retrieve agent from files backend api
        let uri = format!("/api/agents/{agent_id}");
        let result: Result<AsyncResponse, RequestError> = self
            .make_async_request("GET", &uri, None, Some(auth_token.to_string()))
            .await;

        // fail if couldn't send request
        if result.is_err() {
            return Err(RequestError {
                code: 415,
                message: result.unwrap_err().message,
                http_code: Some(500),
            });
        }
        let response = result.unwrap();

        // fail if response is not 2xx
        if !response.status().is_success() {
            return Err(RequestError {
                code: 416,
                message: response.status().to_string(),
                http_code: Some(response.status().as_u16()),
            });
        }
        let result_str = response.text().await.unwrap();

        // deserialize agent
        let deserialize_result = serde_json::from_str(&result_str);
        if deserialize_result.is_err() {
            return Err(RequestError {
                code: 417,
                message: deserialize_result.unwrap_err().to_string(),
                http_code: Some(500),
            });
        }
        let agent: Agent = deserialize_result.unwrap();

        Ok((agent, auth_token.to_string()))
    }

    pub async fn check_user_auth(
        &self,
        user_id: u32,
        auth_cookie: Option<&Cookie<'_>>,
    ) -> Result<(), RequestError> {
        // fail if cannot unwrap cookie value
        if auth_cookie.is_none() {
            return Err(RequestError {
                code: 424,
                message: "".to_string(),
                http_code: Some(401),
            });
        }
        let auth_token = auth_cookie.unwrap().value();

        // retrieve user from Files backend api
        let uri = format!("/api/users/{user_id}");
        let result: Result<AsyncResponse, RequestError> = self
            .make_async_request("GET", &uri, None, Some(auth_token.to_string()))
            .await;

        // fail if couldn't send request
        if result.is_err() {
            return Err(RequestError {
                code: 425,
                message: result.unwrap_err().message,
                http_code: Some(500),
            });
        }
        let response = result.unwrap();

        // fail if response is not 2xx
        if !response.status().is_success() {
            return Err(RequestError {
                code: 426,
                message: response.status().to_string(),
                http_code: Some(response.status().as_u16()),
            });
        }

        Ok(())
    }

    pub async fn send_upload_status_update_async(&self, transfer: &Transfer, message: &str) {
        let uri = format!(
            "/api/agent/{}/transfers/{}/update/{message}",
            transfer.agent_id, transfer.transfer_id
        );

        let auth = Some(transfer.rc_auth.to_string());
        let _ = self.make_async_request("PATCH", &uri, None, auth).await;

        ()
    }

    pub fn send_upload_status_update(&self, transfer: &Transfer, message: &str) {
        let uri = format!(
            "/api/agent/{}/transfers/{}/update/{message}",
            transfer.agent_id, transfer.transfer_id
        );

        let auth = Some(transfer.rc_auth.to_string());
        let _ = self.make_request("PATCH", &uri, None, auth);

        ()
    }

    pub fn get_local_resource(&self, user_id: u32, path: &str) -> Result<String, ClientError> {
        let uri = format!("/api/agent/{user_id}/resources/{path}");

        let mut response = match self.make_request("GET", &uri, None, None) {
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

        let mut response = match self.make_request("POST", &uri, Some(items), None) {
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

        let mut response = match self.make_request("POST", uri, Some(request), None) {
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
        let mut response = match self.make_request("GET", "/api/version", None, None) {
            Ok(r) => r,
            Err(_e) => return "unknown".to_string(),
        };

        let mut version = String::new();
        return match response.read_to_string(&mut version) {
            Ok(_) => version,
            Err(_) => "unknown".to_string(),
        };
    }

    fn make_request(
        &self,
        method: &str,
        uri: &str,
        body: Option<String>,
        auth_token: Option<String>,
    ) -> Result<Response, RequestError> {
        let request_url = self.request_url(uri);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        let mut req = match method {
            "GET" => client.get(request_url),
            "DELETE" => client.delete(request_url),
            "PATCH" => client.patch(request_url),
            "POST" => client.post(request_url),
            _ => {
                return Err(RequestError {
                    code: 338,
                    message: format!("Invalid request method: {method}"),
                    http_code: Some(500),
                })
            }
        };

        if body.is_some() {
            req = req
                .header("Content-Type", "application/json")
                .body(body.unwrap());
        }

        if auth_token.is_some() {
            if method == "GET" {
                req = req.header("Cookie", format!("auth={}", auth_token.unwrap()));
            } else {
                req = req.header("X-Auth", format!("{}", auth_token.unwrap()));
            }
        }

        match req.send() {
            Ok(r) => Ok(r),
            Err(e) => Err(RequestError {
                code: 349,
                message: e.to_string(),
                http_code: Some(500),
            }),
        }
    }

    async fn make_async_request(
        &self,
        method: &str,
        uri: &str,
        body: Option<String>,
        auth_token: Option<String>,
    ) -> Result<AsyncResponse, RequestError> {
        let request_url = self.request_url(uri);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        let mut req = match method {
            "GET" => client.get(request_url),
            "DELETE" => client.delete(request_url),
            "PATCH" => client.patch(request_url),
            "POST" => client.post(request_url),
            _ => {
                return Err(RequestError {
                    code: 348,
                    message: format!("Invalid request method: {method}"),
                    http_code: Some(500),
                })
            }
        };

        if body.is_some() {
            req = req
                .header("Content-Type", "application/json")
                .body(body.unwrap());
        }

        if auth_token.is_some() {
            if method == "GET" {
                req = req.header("Cookie", format!("auth={}", auth_token.unwrap()));
            } else {
                req = req.header("X-Auth", format!("{}", auth_token.unwrap()));
            }
        }

        match req.send().await {
            Ok(r) => Ok(r),
            Err(e) => Err(RequestError {
                code: 349,
                message: e.to_string(),
                http_code: Some(500),
            }),
        }
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
