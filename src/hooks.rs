use iron;
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use itertools::join;

#[macro_use]
use serde_derive;

use serde_json;

use logging::HasLogger;
use post::{PostH, PostError};

use std::ops::DerefMut;
use std::collections::BTreeMap;
use std::{convert, error, fmt, clone};
use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Deserialize)]
pub struct Shared {
    pub data: Arc<Mutex<Vec<String>>>,
}
//vv-for Deserializing the incoming request-vv//
#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
  pull_request: PullRequestObject,
}
#[derive(Debug, Deserialize)]
pub struct PullRequestObject {
  repo: RepoObject,
}
#[derive(Debug, Deserialize)]
pub struct RepoObject {
  id: i32,
  url: String,
}

// maybe not "new" but something referencing shared?
// 
// impl PullRequestEvent {
//   pub fn new() -> PullRequestEvent {
//     PullRequestEvent {
//
//     }
//   }
// }

//vv-for the incoming request-vv//
#[derive(Debug, Deserialize)]
pub struct HookH {
  shared_data: Shared
}
impl HookH {
  pub fn new(s: Shared) -> HookH {
    HookH { 
      shared_data: s,
    }
  }
}

//taken from your utils.rs module as an example/template
impl Handler for HookH {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut data = String::new();
        if let Err(_) = req.body.read_to_string(&mut data) {
            return Err(IronError::new(PostError::ParseError, (
                status::BadRequest,
                "Failed reading POST request body to \
                 'data' buffer.",
            )));
        };
        println!("Deserializing payload: {}", &data);
        let webhook: PullRequestEvent = match serde_json::from_str(&mut data) {
            Ok(thing) => thing,
            Err(e) => {
                println!("{:?}", &e);
                return Err(IronError::new(PostError::ParseError, (
                    status::BadRequest,
                    "Failed reading POST request body to \
                     'data' buffer.",
                )));
            },
        };
        // let keys: &[&str] = &["pull_request", "merged_at"];
        let merged_status = webhook.pull_request.repo.url;
        if merged_status != "False" {
            println!("PR hasn't been merged yet!");
            return Err(IronError::new(PostError::ParseError, (
                status::BadRequest,
                "Failed reading POST request body to \
                 'data' buffer.",
            )));
        } else {
            println!("merged_at: {}", merged_status);
            return Ok(Response::with((status::Ok, "PR Merged, Building...")));
        }
    }
}
