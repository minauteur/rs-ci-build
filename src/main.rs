//! Main
//!
//! This contains the main function for handling http GET requests containing
//! a url-encoded "path" query specifying remote directory for running 'cargo build'.
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate rustc_serialize;
extern crate hyper;
extern crate iron;
extern crate router;
extern crate mustache;
extern crate rustache;
extern crate crypto;
extern crate chrono;
extern crate avro;

extern crate rusqlite;

extern crate rs_build;

use iron::prelude::*;
use router::Router;

use chrono::prelude::*;
use rs_build::database;
use rs_build::build;
use rs_build::config::Config;
use rs_build::hooks::HookH;
use rs_build::post::{DataH, PostH};
use rs_build::reports::ReportH;
use rs_build::timestamp;
use rs_build::logging::HasLogger;
use std::sync::{Arc, Mutex};
use std::env;

static REPO_CONFIG_LOC: &str = "/home/minauteur/Litmus/litmus-ci/repo_config.toml";

fn main() {
    let key = "SECRET_TOKEN";
    match env::var(key) {
        Ok(val) => println!("{}: {:?}", key, val),
        Err(e) => println!("couldn't interpret {}: {}", key, e),
    }

    let utc: DateTime<UTC> = UTC::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    let local: DateTime<Local> = Local::now(); // e.g. `2014-11-28T21:45:59.324310806+09:00`

    let share = Arc::new(Mutex::new(Vec::new()));
    let reports = Arc::new(Mutex::new(Vec::new()));

    let config = match Config::new(REPO_CONFIG_LOC) {
        Ok(btree) => btree,
        Err(e) => panic!("Config Error! {}", &e),
    };

    let arc_cfg = Arc::new(config.repositories.clone());

    let mut repo_list = Vec::new();
    println!("listing local repositories:");
    for (name, address) in config.repositories {
        println!("{} on file at: {}", &name, &address);
        repo_list.push(name);
    }

    let post_h = PostH::new(share.clone());
    let data_h = DataH::new(share.clone());
    let hook_h = HookH::new(arc_cfg.clone(), reports.clone());

    let report_h = ReportH::new(arc_cfg.clone(), reports.clone());

    let mut router = Router::new();

    router.get("/reports", report_h, "reports");
    router.get("/build", build::build_h, "build");
    router.get("/data", data_h, "data");
    router.post("/post", post_h, "post");
    router.post("/hook", hook_h, "hook");
    Iron::new(router).http("localhost:8080").unwrap();
}
