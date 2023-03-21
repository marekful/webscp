#[macro_use] extern crate rocket;

use std::process::{Command, Stdio};
use rocket::serde::{Deserialize, json::Json, Serialize};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct HostInfo<'r> {
    host: &'r str,
    port: &'r str,
    secret: Option<&'r str>
}

#[derive(Serialize, Debug)]
struct RegisterPublicKeyResponse {
    success: Option<bool>,
    error: Option<String>,
    code: Option<i32>
}

#[derive(Serialize, Debug)]
struct VersionResponse {
    version: Option<String>,
    error: Option<String>,
    latency: Option<String>,
}

#[derive(Serialize, Debug)]
struct PingResponse {
    latency: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Debug)]
struct ResourcesResponse {
    resource: Option<String>,
    error: Option<String>,
}

#[post("/register-public-key", data = "<host_info>")]
fn register_public_key(host_info: Json<HostInfo<'_>>) -> Json<RegisterPublicKeyResponse> {
    // attempt to execute 'exchange-keys' client command
    let result = Command::new("target/debug/cli")
        .arg("exchange-keys")
        .arg(host_info.host)
        .arg(host_info.port)
        .arg(host_info.secret.unwrap())
        .stdout(Stdio::piped())
        .output();

    // return error response if failed to execute command
    if result.is_err() {
        return Json(RegisterPublicKeyResponse {
            error: Some(result.unwrap_err().to_string()),
            code: Some(499),
            success: None
        })
    }
    let output = result.unwrap();

    // return error response if the command's error output is not empty
    let stderr = String::from_utf8(output.stderr).unwrap();
    if stderr.len() > 0 {
        return Json(RegisterPublicKeyResponse {
            error: Some(String::from(stderr.trim())),
            code: Some(output.status.code().unwrap()),
            success: None
        })
    }

    // return success response
    Json(RegisterPublicKeyResponse {
        success: Some(output.status.success()),
        error: None,
        code: None
    })
}

#[get("/version/<host>/<port>")]
fn version(host: &str, port: &str) -> Json<VersionResponse> {

    // attempt to execute 'remote-version' client command
    let version_result = Command::new("target/debug/cli")
        .arg("get-remote-version")
        .arg(host)
        .arg(port)
        .stdout(Stdio::piped())
        .output();

    // return error response if failed to execute command
    if version_result.is_err() {
        return Json(VersionResponse {
            latency: None,
            version: None,
            error: Some(version_result.unwrap_err().to_string()),
        })
    }
    let version_result = version_result.unwrap();

    // return error response if the command's error output is not empty
    let version_error = String::from_utf8(version_result.stderr).unwrap();
    if version_error.len() > 0 {
        return Json(VersionResponse {
            latency: None,
            version: None,
            error: Some(version_error.trim().to_string()),
        })
    }

    // attempt to execute 'ping' client command
    let ping_result = Command::new("target/debug/cli")
        .arg("ping")
        .arg(host)
        .arg(port)
        .stdout(Stdio::piped())
        .output();

    // return error response if failed to execute command
    if ping_result.is_err() {
        return Json(VersionResponse {
            latency: None,
            version: None,
            error: Some(ping_result.unwrap_err().to_string()),
        })
    }
    let ping_result = ping_result.unwrap();

    // return error response if the command's error output is not empty
    let ping_error = String::from_utf8(ping_result.stderr).unwrap();
    if ping_error.len() > 0 {
        return Json(VersionResponse {
            latency: None,
            version: None,
            error: Some(ping_error.trim().to_string()),
        })
    }

    // return success response
    let ping_output = String::from_utf8(ping_result.stdout).unwrap();
    let version_output = String::from_utf8(version_result.stdout).unwrap();

    Json(VersionResponse {
        latency: Some(ping_output.trim().to_string()),
        version: Some(version_output.trim().to_string()),
        error: None,
    })
}

#[get("/ping/<host>/<port>")]
fn ping(host: &str, port: &str) -> Json<PingResponse> {

    // attempt to execute 'ping' client command
    let result = Command::new("target/debug/cli")
        .arg("ping")
        .arg(host)
        .arg(port)
        .stdout(Stdio::piped())
        .output();

    // return error if failed to execute command
    if result.is_err() {
        return Json(PingResponse {
            latency: None,
            error: Some(result.unwrap_err().to_string()),
        })
    }
    let result = result.unwrap();

    // return error if the command's error output is not empty
    let stderr = String::from_utf8(result.stderr).unwrap();
    if stderr.len() > 0 {
        return Json(PingResponse {
            latency: None,
            error: Some(stderr.trim().to_string()),
        })
    }

    // Return success
    let output = String::from_utf8(result.stdout).unwrap();

    Json(PingResponse{
        latency: Some(output.trim().to_string()),
        error: None,
    })
}

#[get("/resources/<host>/<port>/<path>")]
fn resources(host: &str, port: &str, path: &str) -> Json<ResourcesResponse> {

    // attempt to execute 'get-remote-resource' client command
    let result = Command::new("target/debug/cli")
        .arg("get-remote-resource")
        .arg(host)
        .arg(port)
        .arg(path)
        .stdout(Stdio::piped())
        .output();

    // return error response if failed to execute command
    if result.is_err() {
        return Json(ResourcesResponse {
            resource: None,
            error: Some(result.unwrap_err().to_string()),
        })
    }
    let result = result.unwrap();

    // return error response if the command's error output is not empty
    let stderr = String::from_utf8(result.stderr).unwrap();
    if stderr.len() > 0 {
        return Json(ResourcesResponse {
            resource: None,
            error: Some(stderr.trim().to_string()),
        })
    }
    // return success response
    let stdout = String::from_utf8(result.stdout).unwrap();
    return Json(ResourcesResponse{
        resource: Some(format!("{}", stdout)),
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
        .mount("/api", routes![resources])
        .mount("/api", routes![version])
        .mount("/api", routes![hello])
        .launch()
        .await?;

    Ok(())
}
