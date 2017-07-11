//! Main
//!
//! This contains the main function for handling http GET requests containing
//! a url-encoded "path" query specifying remote directory for running 'cargo build'.
#[macro_use]
extern crate serde_derive;
extern crate iron;
extern crate router;
extern crate litmus_ci;

use iron::prelude::*;
use router::Router;

use litmus_ci::build;
use litmus_ci::hooks::{HookH};
use litmus_ci::post::{DataH, PostH};
use litmus_ci::config::Config;

use std::sync::{Arc, Mutex};

// static REPO_CONFIG_LOC: &'static str = "/home/minauteur/Litmus/rs-ci-build/repo_config.toml";

fn main() {
    let mut share = Arc::new(Mutex::new(Vec::new()));

    // let config = Config::new(REPO_CONFIG_LOC.to_string());
    // for (key, values) in config.repositories {
    //     println!("\nrepo: {}, location on disk: {}", &key, &values);
    // }     

    let post_h = PostH::new(share.clone());
    let data_h = DataH::new(share.clone());
    let hook_h = HookH::new(share.clone());
    

    let mut router = Router::new();

    router.get("/build", build::build_h, "build");
    router.get("/data", data_h, "data");
    router.post("/post", post_h, "post");
    router.post("/hook", hook_h, "hook");
    Iron::new(router).http("localhost:8080").unwrap();
}
