use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;

use serde_json;

use logging::HasLogger;
use post::PostError;
use config::Config;

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

static REPO_CONFIG_LOC: &'static str = "/home/minauteur/Litmus/rs-ci-build/repo_config.toml";

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
pub struct HookH {
    shared_data: Arc<Mutex<Vec<String>>>,
}
impl HookH {
    pub fn new(mut s: Arc<Mutex<Vec<String>>>) -> HookH {
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
        let name = webhook.repository.name.clone();
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
        info!(logger, "\n'repo': {},\n'url': {},\n'merged_at': {},\n", name, target_url, merged);
        let target_url_msg = format!("\n'repo': {},\n'url': {},\n'merged_at': {},\n",
            name, target_url, merged);
        let mut shared = match self.shared_data.lock() {
            Ok(vec) => vec,
            Err(e) => {
                println!("{:?}", &e);
                return Err(IronError::new(PostError::MutexError, (
                    status::BadRequest,
                    "Mutex Error: couldn't get shared data!",
                )));
            },
        };
        let config = Config::new(REPO_CONFIG_LOC.to_string());
        for (repo, address) in config.repositories {
            if repo.to_string() == webhook.repository.name.as_str() {
                println!("\n{}, present and accounted for", &repo);
            }
            println!("\nrepo: {}, location on disk: {}", &repo, &address);
            
        }
        shared.deref_mut().push(webhook.repository.name);
        return Ok(Response::with((status::Ok, target_url_msg))); 
    }
}
