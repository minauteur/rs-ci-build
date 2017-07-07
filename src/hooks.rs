use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;

use serde_json;

use logging::HasLogger;
use post::PostError;

use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Deserialize)]
//FIXME: I'll go back to using Arc<Mutex<Vec<String>>> directly before the next commit.
pub struct Shared {
    pub data: Arc<Mutex<Vec<String>>>,
}

//for Deserializing
#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pull_request: PullRequestObject,
    repository: RepositoryObject,
}
#[derive(Debug, Deserialize)]
pub struct PullRequestObject {
    merged_at: serde_json::value::Value,
}
#[derive(Debug, Deserialize)]
pub struct RepositoryObject {
    id: i32,
    name: String,
    html_url: String,
}

//handling the incoming request
#[derive(Debug, Deserialize)]
pub struct HookH {
    shared_data: Shared,
}
impl HookH {
    pub fn new(s: Shared) -> HookH {
        HookH { shared_data: s }
    }
}

impl Handler for HookH {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let logger = req.get_logger();
        let mut data = String::new();
        if let Err(_) = req.body.read_to_string(&mut data) {
            return Err(IronError::new(PostError::ParseError, (
                status::BadRequest,
                "Failed reading POST request to buffer!",
            )));
        };
        // info!(logger, "POST Received... Deserializing payload."; "payload" => &data);
        println!("\nPOST received. Deserializing...");
        let webhook: PullRequestEvent = match serde_json::from_str(&mut data) {
            Ok(thing) => thing,
            Err(e) => {
                println!("{:?}", &e);
                return Err(IronError::new(PostError::ParseError, (
                    status::BadRequest,
                    "Failed deserializing payload!",
                )));
            },
        };
        let merged = webhook.pull_request.merged_at;
        let target_url = webhook.repository.html_url;
        match merged.as_str() {
            Some(string) => {
                string
            },
            None => {
                info!(logger, "\nRepository at {} hasn't been merged yet!", &target_url);
                let merge_null_msg = format!("\n'merged_at': null,\n'repo':...'url': {}\n",
                    target_url);
                return Err(IronError::new(PostError::ParseError, (merge_null_msg)));
            }
        };
        info!(logger, "\n'merged_at': {}\n'repository':...'url': {}", merged, target_url);
        let target_url_msg = format!("\n'merged_at': {},\n'repo':...'url': {}\n",
            merged, target_url);
        return Ok(Response::with((status::Ok, target_url_msg))); 
    }
}
