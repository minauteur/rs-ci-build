//! Build Module
//!
//! This module contains the logic for a url-encoded "path" query
//! for automating 'cargo build' remotely.

use errors::CIError;
use iron::prelude::*;
use iron::status;

use logging::HasLogger;

use std::process::Command;

use urlencoded::{UrlDecodingError, UrlEncodedQuery};

//handler function, called from main.rs upon http GET request to route /build
pub fn build_h(req: &mut Request) -> IronResult<Response> {
    //attaches .get_logger() method from 'logging.rs' to request object
    let logger = req.get_logger();
    //on an Ok result, match req.get::<UrlEncodedQuery> yields a HashMap containing
    //url parameters (keys) and their associated values (stored as entries).
    let url_query_map = match req.get::<UrlEncodedQuery>() {
        Ok(hashmap) => hashmap,
        Err(why) => {
            error!(logger, "\nreq.get::UrlEncodedQuery>() \
                failed to return UrlQueryMap"; "reason" => %why);
            let err_resp = format!("\nCould not extract HashMap from url...\nError: {}\n", &why);
            return Err(IronError::new(CIError::UrlDecoding(UrlDecodingError::EmptyQuery),
                                      (status::BadRequest, err_resp)));
        }
    };
    //Now that we have our HashMap, Option to extract the 'path' (key/parameter) and its associated
    //values before storing them in a vector via an iterator
    let path = match url_query_map
              .get("path")
              .and_then(|path_vec| path_vec.iter().next()) {
        Some(path_value) => path_value,
        None => {
            info!(logger, "\nNo value for 'path' key in UrlQueryMap!\n");
            let no_val = format!("\nProblem unpacking urlencoded HashMap: No value(s) exist \
                                      for given key!\n");
            return Err(IronError::new(CIError::UrlDecoding(UrlDecodingError::EmptyQuery),
                                      (status::BadRequest, no_val)));
        }
    };
    //format preamble String in preparation for writing command output to http response body
    let mut resp = format!("\nRunning 'cargo build'...\nTarget Directory: {}\n", &path);
    let output: String = match Command::new("cargo")
              .arg("build")
              .current_dir(path)
              .output() {
        Ok(info) => {
            //Ok result reads from stderr because 'cargo build' outputs to stderr by default.
            let info_str = String::from_utf8_lossy(&info.stderr);
            info!(logger, "\nSuccess!\n"; "stdout" => %info_str);
            format!("\nSuccess!\noutput: {}\n", info_str)
        }
        Err(e) => {
            error!(logger, "\n'cargo build' failed!\n"; "reason"=> %e);
            format!("\n'cargo build' failed!\nReason: {}\n", e.to_string())
        }
    };
    //now that all of the component pieces of our possible http response are stored in 'output',
    //push them to resp and send away!
    resp.push_str(&output);
    Ok(Response::with((status::Ok, resp)))
}
