use rocket::{
    http::{CookieJar, Status},
    State,
};

use crate::{CancelTransferRequests, Files};

#[delete("/agents/<agent_id>/transfers/<transfer_id>")]
pub async fn cancel_transfer(
    agent_id: u32,
    transfer_id: &str,
    files: &State<Files>,
    cancel_requests: &State<CancelTransferRequests>,
    cookies: &CookieJar<'_>,
) -> Status {
    // verify that the requester has a valid session in Files and owns the referred agent
    let (_, _) = match files.api.get_agent(agent_id, cookies.get("rc_auth")).await {
        Ok(a) => a,
        Err(_) => {
            return Status::Forbidden;
        }
    };

    // get the shared state holding the list of cancel transfer request flags
    let transfers = cancel_requests.transfers.lock().unwrap();

    // check if the referred transfer is registered in the list
    if let Some(cancel_request) = transfers.get(transfer_id) {
        // lock the mutex and get the flag for r/w
        let mut signal_cancel = cancel_request.lock().unwrap();

        // set the flag
        return if !*signal_cancel {
            *signal_cancel = true;

            Status::Ok
        } else {
            Status::NotModified
        };
    }

    // success response
    Status::NotFound
}
