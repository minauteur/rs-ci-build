//! Main
//!
//! This contains the main function for handling http GET requests containing
//! a url-encoded "path" query specifying remote directory for running 'cargo build'.
extern crate iron;
extern crate router;
extern crate ci_build;

use ci_build::build;
//use ci_build::logging;

use iron::prelude::*;
use router::Router;

fn main() {
    let mut router = Router::new();

    router.get("/build", build::build_h, "build");
    Iron::new(router).http("127.0.0.1:8080").unwrap();
}
