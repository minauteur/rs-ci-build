//! Webhooks module.
//!
//! Defines structs for deserializing Github webhook payloads into usable Rust values.

use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;

use config::ConfigError;
use logging::HasLogger;
use build::Build;
use post::PostError::{MutexError, ParseError};
use reports::Report;
use serde_json;

use std::env;
use std::collections::BTreeMap;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use timestamp::Timestamp;


//for Deserializing
#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pull_request: PullRequestObject,
    repository: RepositoryObject,
}
#[derive(Debug, Deserialize)]
pub struct PullRequestObject {
    merged_at: serde_json::value::Value,
    url: String,
    diff_url: String,
    title: String,
    user: UserObject,
}
#[derive(Debug, Deserialize)]
pub struct RepositoryObject {
    id: i32,
    name: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct UserObject {
    login: String,
    html_url: String,
}

pub struct HookH {
    subjects: Arc<BTreeMap<String, String>>,
    reports_to: Arc<Mutex<Vec<Report>>>,
}
impl HookH {
    pub fn new(listing: Arc<BTreeMap<String, String>>, archive: Arc<Mutex<Vec<Report>>>) -> HookH {
        HookH {
            subjects: listing,
            reports_to: archive,
        }
    }
}
// #[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
// pub struct XHubSignature (String);

// impl Header for XHubSignature {
//     fn header_name() -> &'static str {
//         "X-Hub-Signature"
//     }
//     fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<XHubSignature> {
//            if keys.len() != 1 {
//         Err(hyper::Error::Header)
//     }
//     else return raw[0]
// }

//handling the incoming request
impl Handler for HookH {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let logger = req.get_logger();

        let token: String = match env::var("SECRET_TOKEN").ok() {
            Some(s) => s,
            None => {
                return Err(IronError::new(env::VarError::NotPresent, (status::InternalServerError, "Something went wrong")))
            }
        };

        info!(logger, "Validating request...");

        // println!("{:?}", req.headers.get());

        let mut data = String::new();
        if let Err(_) = req.body.read_to_string(&mut data) {
            return Err(IronError::new(ParseError, (
                status::BadRequest,
                "Failed reading POST request to \
                 buffer!",
            )));
        };

        info!(logger, "POST Received... Deserializing payload.");

        let webhook: PullRequestEvent = match serde_json::from_str(&mut data) {
            Ok(thing) => thing,
            Err(e) => {
                println!("{:?}", &e);
                return Err(IronError::new(ParseError, (
                    status::BadRequest,
                    "Failed deserializing payload!",
                )));
            },
        };

        let time_filed = Timestamp::now();

        match webhook.pull_request.merged_at.as_str() {
            Some(string) => string,
            None => {
                info!(
                    logger,
                    "Repository at {} hasn't been merged yet!",
                    &webhook.repository.html_url
                );
                let merge_null_msg = format!(
                    "\n'merged_at': null, 'repo': {}, 'url': {}",
                    &webhook.repository.name,
                    &webhook.repository.html_url
                );
                return Err(IronError::new(ParseError, (merge_null_msg)));
            },
        };
        let cargo_msg_placeholder = "nothing here yet".to_string();
        let mut res =
            format!(
            "\n'repo': {}, 'ID': {}, 'url': {}, 'merged_at': {}, 'cargo_msg':{}",
            webhook.repository.name,
            webhook.repository.id,
            webhook.repository.html_url,
            webhook.pull_request.merged_at,
            cargo_msg_placeholder,
        );
        let mut into_archive = match self.reports_to.lock() {
            Ok(vec) => vec,
            Err(e) => {
                println!("{:?}", &e);
                return Err(IronError::new(MutexError, (
                    status::InternalServerError,
                    "Mutex Error: couldn't get lock!",
                )));
            },
        };

        let name = webhook.repository.name.clone();

        if self.subjects.deref().contains_key(&name) {
            let path = match self.subjects.get(&name) {
                Some(path) => path,
                None => panic!(ConfigError::BadConfig),
            };
            let cmd = Build::new(path.clone());
            let msg = cmd.to_string();
            // let pb = PathBuf::from(&path);
            // let cmd: String = match Command::new("cargo").arg("build").current_dir(pb).output() {
            //     Ok(info) => {
            //         //'cargo build' outputs to stderr by default.
            //         let msg = String::from_utf8_lossy(&info.stderr);
            //         info!(logger, "Success!"; "stdout" => %msg);
            //         msg.trim().to_string()
            //     },
            //     Err(e) => {
            //         error!(logger, "'cargo build' failed!"; "err" => %e);
            //         format!("'cargo build' failed! Error: {}", e.to_string())
            //     },
            // };
            // // let info = serde_json::to_string
            let report = Report {
                repo_name: webhook.repository.name.clone(),
                repo_url: webhook.repository.html_url.clone(),
                pr_url: webhook.pull_request.url.clone(),
                diff_url: webhook.pull_request.diff_url.clone(),
                user: webhook.pull_request.user.login.clone(),
                user_url: webhook.pull_request.user.html_url.clone(),
                title: webhook.pull_request.title.clone(),
                time: time_filed.timestamp,
                cargo_msg: msg.clone(),
            };
            let content =
                format!(
                "\n'repo_name': {}, 'repo_id': {}, 'html_url': {}, \
                'merged_at': {}, 'loc_path': {}, 'cargo_msg':{}",
                webhook.repository.name.clone(),
                webhook.repository.id.clone(),
                webhook.repository.html_url.clone(),
                webhook.pull_request.merged_at.clone(),
                path,
                msg,
            );
            res.push_str(&content);
            into_archive.deref_mut().push(report);
            return Ok(Response::with((status::Ok, res)));
        }
        return Ok(Response::with((status::Ok, res)));
    }
}
