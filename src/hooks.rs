use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;

use serde_json;

use logging::HasLogger;
use post::PostError;
use config::Config;
use std::path::PathBuf;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use std::process::Command;

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
                info!(logger, "Repository at {} hasn't been merged yet!", &target_url);
                let merge_null_msg = format!("\n'merged_at': null, 'repo': {}, 'url': {}",
                    name, target_url);
                return Err(IronError::new(PostError::ParseError, (merge_null_msg)));
            }
        };
        // info!(logger, "\n'repo': {}, 'url': {}, 'merged_at': {},", name, target_url, merged);
        let mut target_url_msg = format!("\n'repo': {}, 'url': {}, 'merged_at': {}, ",
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

        shared.deref_mut().push(webhook.repository.name.clone());

        let config = Config::new(REPO_CONFIG_LOC.to_string());
        
        println!("Listing Repositories...");
        
        for (repo, address) in config.repositories {
            //first what's in the config file?
            println!("\nrepo: {}, location on disk: {}", &repo, &address);
            //match that to the name in the payload
            if repo.to_string() == webhook.repository.name.as_str() {
                //if we get a match, print, trigger the build, and move on to the next match.
                info!(logger, "Attempting to 'cargo build' {} in {}", &repo, &address);
                let mut resp = format!("\n'cargo build' ran in {}", &address);
                let output: String = match Command::new("cargo")
                    .arg("build")
                    .current_dir((PathBuf::from(&address.clone())))
                    .output() {
                        Ok(info) => {
                            //Ok result reads from stderr because 'cargo build' outputs to stderr by default.
                            let mut info_str = String::from_utf8_lossy(&info.stderr);
                            info!(logger, "Success!"; "stdout" => %info_str);
                            format!("\nSuccess! stdout: {}", info_str)
                        },
                        Err(e) => {
                            error!(logger, "'cargo build' failed!"; "reason"=> %e);
                            format!("\n'cargo build' failed! Reason: {}", e.to_string())
                        },
                };
                //'output' captures what we want in our http response, push it to resp after building.
                resp.push_str(&output);
                return Ok(Response::with((status::Ok, resp)))
            }
        }
        //if there's no repo match, then return the deserialized info from payload in an Ok.
        return Ok(Response::with((status::Ok, target_url_msg))); 
    }
}
