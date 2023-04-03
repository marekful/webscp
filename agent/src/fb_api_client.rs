use reqwest::{blocking::Response, Response as AsyncResponse};
use std::env;

use crate::constants::DEFAULTS;

pub struct RequestError {
    pub code: i32,
    pub message: String,
    pub http_code: Option<u16>,
}

fn get_base_url() -> String {
    let default_fb_api_address = DEFAULTS.default_fb_api_address;
    let fb_api_address_result = env::var(DEFAULTS.env_name_fb_api_address);
    return fb_api_address_result.unwrap_or(default_fb_api_address.to_string());
}

pub async fn send_upload_status_update_async(transfer_id: &str, message: &str) {
    let uri = format!("/api/sse/transfers/{transfer_id}/update/{message}");

    let _ = make_async_get_request(uri).await;

    ()
}

pub fn send_upload_status_update(transfer_id: &str, message: &str) {
    let uri = format!("/api/sse/transfers/{transfer_id}/update/{message}");

    let _ = make_get_request(uri);

    ()
}

async fn make_async_get_request(uri: String) -> Result<AsyncResponse, RequestError> {
    let base_url = get_base_url();
    let request_url = format!("{base_url}{uri}");

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

fn make_get_request(uri: String) -> Result<Response, RequestError> {
    let base_url = get_base_url();
    let request_url = format!("{base_url}{uri}");

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
