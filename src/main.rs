//! Main
//!
//! This contains the main function for handling http GET requests containing
//! a url-encoded "path" query specifying remote directory for running 'cargo build'.
extern crate iron;
extern crate router;
extern crate h_struct;

use std::sync::{Mutex, Arc};
use h_struct::build;
use h_struct::post::{PostH, DataH};
use iron::prelude::*;
use router::Router;

fn main() {
    let post_h = PostH::new(Arc::new(Mutex::new(Vec::new())));
    let data_h = DataH::new(post_h.clone());
    let mut router = Router::new();

    router.get("/build", build::build_h, "build");
    router.get("/data", data_h, "data");
    router.post("/post", post_h, "post");
    Iron::new(router).http("localhost:8080").unwrap();
}
