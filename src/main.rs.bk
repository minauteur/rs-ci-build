//! Main
//!
//! This contains the main function for handling http GET requests containing
//! a url-encoded "path" query specifying remote directory for running 'cargo build'.
#[macro_use] extern crate serde_derive;
extern crate iron;
extern crate router;
extern crate litmus_ci;

use iron::prelude::*;
use litmus_ci::build;
use litmus_ci::hooks::{Shared, HookH};
use litmus_ci::post::{DataH, PostH};
use router::Router;
use std::sync::{Arc, Mutex};

fn main() {
    let share = Shared { data: Arc::new(Mutex::new(Vec::new()))};
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
