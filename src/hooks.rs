use iron;
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use itertools::join;

#[macro_use]
use serde_derive;

use serde_json;

use logging::HasLogger;
use post::{PostError, PostH};

use std::{clone, convert, error, fmt};
use std::collections::BTreeMap;
use std::io::Read;
use std::ops::DerefMut;
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
}
#[derive(Debug, Deserialize)]
pub struct PullRequestObject {
    repo: RepoObject,
    merged_at: String,
}
#[derive(Debug, Deserialize)]
pub struct RepoObject {
    id: i32,
    url: String,
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
        println!("\nPOST received. Deserializing...\n");
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
        let target_url = webhook.pull_request.repo.url;
        if !merged {
            info!(logger, "\n'merged_at': false\n");
            let no_merge_msg = format!("\n'merged_at': false,\n'repo':...'url': {}\n",
                target_url);
            return Err(IronError::new(PostError::ParseError, (no_merge_msg)));
        } else {
            info!(logger, "merged_at: {}", &merged);
            let target_url_msg = format!("\n'merged_at': {},\n'repo':...'url': {}\n", 
                merged, target_url);
            return Ok(Response::with((status::Ok, target_url_msg)));
        }
    }
}
