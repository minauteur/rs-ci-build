use iron;
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;

use serde_json;

use std::{convert, error, fmt};
use std::ops::DerefMut;
use std::io::Read;
use std::sync::{Arc, Mutex};

use logging::HasLogger;

pub struct PostH {
    payload: Arc<Mutex<Vec<String>>>,
}

impl PostH {
    pub fn new(payload: Arc<Mutex<Vec<String>>>) -> PostH {
        PostH { payload: payload }
    }
}

impl Handler for PostH {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut data_buf = String::new();
        let logger = req.get_logger();
        if let Err(_) = req.body.read_to_string(&mut data_buf) {
            error!(logger, "Failed reading request body"; "err" => "Failed reading POST to buffer");
            return Err(IronError::new(PostError::ParseError, (
                status::BadRequest,
                "Failed reading POST request body to \
                 'data' buffer.",
            )));
        };
        let mut post_payload = match self.payload.lock() {
            Ok(vec) => vec,
            Err(e) => {
                error!(logger, "Couldn't get payload from PostH"; "err" => %e);
                return Err(IronError::new(PostError::MutexError, (
                    status::InternalServerError,
                    "Couldn't get payload from PostH",
                )));
            },
        };
        info!(logger, "Got payload from postH!"; "data" => &data_buf);
        post_payload.deref_mut().push(data_buf);
        Ok(Response::with((status::Ok, post_payload.join(" ;\n"))))
    }
}

pub struct DataH {
    response: Arc<Mutex<Vec<String>>>,
}
impl DataH {
    pub fn new(res: Arc<Mutex<Vec<String>>>) -> DataH {
        DataH { response: res }
    }
}


impl Handler for DataH {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let logger = req.get_logger();
        let mut html = match self.response.lock() {
            Ok(vec) => vec,
            Err(e) => {
                error!(logger, "Couldn't get payload from DataH"; "err" => %e);
                return Err(IronError::new(PostError::MutexError, (
                    status::InternalServerError,
                    "Couldn't get payload from DataH",
                )));
            },
        };
        info!(logger, "GET request successful!"; "data" => html.deref_mut().join(";\n"));
        Ok(Response::with((status::Ok, html.join(";\n"))))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostError {
    ParseError,
    MutexError,
}

impl fmt::Display for PostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred.")
    }
}

impl error::Error for PostError {
    fn description(&self) -> &str {
        use self::PostError::*;

        match *self {
            ParseError => "Request could not be understood.",
            MutexError => "Internal server error.",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl convert::From<serde_json::Error> for PostError {
    fn from(_: serde_json::Error) -> PostError {
        PostError::ParseError
    }
}

impl convert::Into<iron::error::IronError> for PostError {
    fn into(self) -> iron::error::IronError {
        use self::PostError::*;
        match self {
            ParseError => {
                IronError::new(self, (
                    status::BadRequest,
                    "I didn't understand your request.",
                ))
            },
            MutexError => {
                IronError::new(self, (status::InternalServerError, "There was a problem."))
            },
        }
    }
}
