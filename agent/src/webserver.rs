#[macro_use] extern crate rocket;

use std::process::{Command, Stdio};
use rocket::serde::{Deserialize, json::Json, Serialize};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct HostInfo<'r> {
    host: &'r str,
    port: &'r str,
    secret: Option<&'r str>
}

#[derive(Serialize)]
struct RegisterPublicKeyResponse {
    success: Option<bool>,
    error: Option<String>,
    code: Option<i32>
}

#[derive(Serialize)]
struct VersionResponse {
    version: Option<String>,
    error: Option<String>,
    latency: Option<String>,
}

#[derive(Serialize)]
struct PingResponse {
    latency: Option<String>,
    error: Option<String>,
}

#[post("/register-public-key", data = "<host_info>")]
fn register_public_key(host_info: Json<HostInfo<'_>>) -> Json<RegisterPublicKeyResponse> {
    // Execute client command
    let output = Command::new("target/debug/client")
        .arg("exchange-keys")
        .arg(host_info.host)
        .arg(host_info.port)
        .arg(host_info.secret.unwrap())
        .stdout(Stdio::piped())
        .output()
        .expect("failed to add public key");

    // Return error
    let stderr = String::from_utf8(output.stderr).unwrap();
    if stderr.len() > 0 {
        return Json(RegisterPublicKeyResponse {
            error: Some(String::from(stderr.trim())),
            code: Some(output.status.code().unwrap()),
            success: None
        })
    }
    // Return success
    Json(RegisterPublicKeyResponse {
        success: Some(output.status.success()),
        error: None,
        code: None
    })
}

#[get("/version/<host>/<port>")]
fn version(host: &str, port: &str) -> Json<VersionResponse> {

    // Execute client command
    let version_output = Command::new("target/debug/client")
        .arg("remote-version")
        .arg(host)
        .arg(port)
        .stdout(Stdio::piped())
        .output()
        .expect("failed to get version");

    // Return error
    let version_error = String::from_utf8(version_output.stderr).unwrap();
    if version_error.len() > 0 {
        return Json(VersionResponse {
            latency: None,
            version: None,
            error: Some(version_error.trim().to_string()),
        })
    }
    // Return success
    let version_result = String::from_utf8(version_output.stdout).unwrap();

    // Execute client command
    let ping_output = Command::new("target/debug/client")
        .arg("ping")
        .arg(host)
        .arg(port)
        .stdout(Stdio::piped())
        .output()
        .expect("failed to get version");

    // Return error
    let ping_error = String::from_utf8(ping_output.stderr).unwrap();
    if ping_error.len() > 0 {
        return Json(VersionResponse {
            latency: None,
            version: None,
            error: Some(ping_error.trim().to_string()),
        })
    }
    // Return success
    let ping_result = String::from_utf8(ping_output.stdout).unwrap();

    Json(VersionResponse {
        latency: Some(String::from(ping_result.trim())),
        version: Some(String::from(version_result.trim())),
        error: None,
    })
}

#[get("/ping/<host>/<port>")]
fn ping(host: &str, port: &str) -> Json<PingResponse> {

    // Execute client command
    let output = Command::new("target/debug/client")
        .arg("ping")
        .arg(host)
        .arg(port)
        .stdout(Stdio::piped())
        .output()
        .expect("failed to get version");

    // Return error
    let stderr = String::from_utf8(output.stderr).unwrap();
    if stderr.len() > 0 {
        return Json(PingResponse {
            latency: None,
            error: Some(stderr.trim().to_string()),
        })
    }
    // Return success
    let stdout = String::from_utf8(output.stdout).unwrap();

    return Json(PingResponse{
        latency: Some(stdout.trim().to_string()),
        error: None,
    })
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, API"
}


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/api", routes![register_public_key])
        .mount("/api", routes![ping])
        .mount("/api", routes![version])
        .mount("/api", routes![hello])
        .launch()
        .await?;

    Ok(())
}
