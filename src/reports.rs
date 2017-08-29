//! Reports module.
//! defines behaviors for serializing/de-derializing build data for UI consumption

use rustc_serialize;

use mustache::{self, MapBuilder};

use std::fmt::{self, Display};
use avro::codec::{ByteStream, AvroCodec};


use hyper::header::{ContentType};
use hyper::mime::{Mime, SubLevel, TopLevel};

use iron::middleware::Handler;
use iron::prelude::*;
use iron::response::Response;
use iron::status;


use post::PostError::MutexError;
use urlencoded::{UrlEncodedQuery};

// use database::SimpleCollection;

use timestamp::{Timestamp};

use logging::HasLogger;

use std::borrow::ToOwned;
use std::collections::{BTreeMap, HashMap};

use std::ops::Deref;

use std::sync::{Arc, Mutex};

static TMPL_LOC: &str = "/home/minauteur/Litmus/litmus-ci/src/tmpl/tmpl.mustache";

//for Deserializing

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, RustcEncodable, PartialOrd, Ord)]
pub struct Report {
    pub repo_name: String,
    pub repo_url: String,
    pub pr_url: String,
    pub diff_url: String,
    pub user: String,
    pub user_url: String,
    pub title: String,
    pub time: i64,
    pub cargo_msg: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, RustcEncodable, PartialOrd, Ord)]
pub struct Reports {
    pub reports: Vec<Report>,
}
struct DBReport<Reports> {
    value: Vec<Reports>,
}

// impl<Reports> Deref for DBReport<Vec<Reports>> {
//     type Target = [Reports];
//     fn deref<'a>(&'a self) -> &'a [Reports] { self[0..].as_slice().unwrap() }
// }
// impl DBReport<Reports> {
//     pub fn to_u8(&mut self) -> Vec<u8> {
        // let mut v: Vec<u8> = Vec::new();
        // v = self.value.deref().to_owned().map(|fields| fields as u8).unwrap();
        // for x in self.reports.iter() {
        // let y = match x.chars().iter().collect() {
        //     Some(c) => c as u8,
        //     u8 => u8,
        // };
        // // u8::from_str_radix() as u8;
        // v.push(y);
        // // v.push(self.clone().as_u8());
        // };
        // v
    // }
//     pub fn to_db(&self, ts: Timestamp) -> HashMap<Vec<u8>, Vec<u8>> {
//         let mut s_c: SimpleCollection = HashMap::new()
//         .insert(ts.encode(), self.to_u8()).unwrap();
//     }
// }
// impl IntoIterator for Reports {
//     type Item = Report;
//     type IntoIter = ::std::vec::IntoIter<Report>
// }

pub struct ReportH {
    subjects: Arc<BTreeMap<String, String>>,
    to_report: Arc<Mutex<Vec<Report>>>,
}
impl ReportH {
    pub fn new(
        matches_tree: Arc<BTreeMap<String, String>>,
        from_archive: Arc<Mutex<Vec<Report>>>,
    ) -> ReportH {
        ReportH {
            subjects: matches_tree,
            to_report: from_archive,
        }
    }
}

//Enum filtering reports to generate
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Select {
    Repo(String),
    Any,
}

impl Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, RustcEncodable)]
pub struct RepoSelOpts {
    name: String,
    selected: bool,
}

impl Handler for ReportH {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        // if req.Url
        let logger = req.get_logger();

        let mut canvas = Vec::new();

        let tmpl = mustache::compile_path(TMPL_LOC).expect("Failed to compile tmpl!");

        let material = MapBuilder::new();

        let repo_list = self.subjects.deref().clone();

        let mut backup_vec: Vec<String> = Vec::new();
        let mut def_map: HashMap<String, Vec<String>> = HashMap::new();

        for (nm, addr) in repo_list.clone().iter() {
            backup_vec.push(nm.clone());
        }
        def_map.insert("repo".to_string(), backup_vec.clone());

        println!("default map value: {:?}", def_map.clone());

        let mut match_vec = Vec::new();

        //SHARED STORAGE LOCK FOR CULLING LOGS
        let logs = match self.to_report.lock() {
            Ok(vec) => vec,
            Err(e) => {
                println!("{:?}", &e);
                return Err(IronError::new(MutexError, (
                    status::InternalServerError,
                    "Mutex Error: couldn't get lock!",
                )));
            },
        };

        //HELPER CLOSURES FOR PRINTING A STRING IF ONE EXISTS OR DEFAULTING OTHERWISE
        let get_if = |s| -> String {
            match s {
                Select::Repo(s) => s,
                Select::Any => repo_list.clone().remove("repositories").unwrap_or_default(),
            }
        };
        let get_any = |s| -> String {
            match s {
                Select::Repo(s) => s,
                Select::Any => "Any".to_string(),
                // repo_list.clone().remove("repositories").unwrap_or_default(),
            }
        };

        //URL QUERY MAP
        let mut url_map = match req.get::<UrlEncodedQuery>().ok() {
            Some(hashmap) => hashmap,
            None => {
                // def_map
                HashMap::new()
            },
        };

        //either return the url-query value for "repos" or invoke Select::Any
        let name = url_map.remove("repo")
                          .and_then(|mut arg_vec| {
            arg_vec.pop().map(|arg| if arg == "Any" {
                Select::Any
            } else {
                Select::Repo(arg)
            })
        })
                          .unwrap_or(Select::Any);
        let num: i32 = url_map.clone()
                              .remove("num")
                              .and_then(|mut num_vec| {
            num_vec.pop().map(|n| n.parse::<i32>().unwrap_or(3))
        })
                              .unwrap_or(3);
        println!("num == {}", &num);

        //need this because .take() asks for usize
        let u: usize = num.clone() as usize;

        let name_cp = get_if(name.clone());

        println!("listing local repositories:");
        for (repos_nm, addr) in repo_list.clone() {

            println!("{} on file at: {}", &repos_nm, &addr);

            if &name_cp.clone() == &repos_nm.to_string() {
                let opts = RepoSelOpts {
                    name: repos_nm.clone(),
                    selected: true,
                };
                match_vec.push(opts);
            } else {
                let opts = RepoSelOpts {
                    name: repos_nm.clone(),
                    selected: false,
                };
                match_vec.push(opts);
            }
        }
        if get_any(name.clone()) == "Any".to_string() {
            let any = RepoSelOpts {
                name: "Any".to_string(),
                selected: true,
            };
            match_vec.push(any);
        } else {
            let any = RepoSelOpts {
                name: "Any".to_string(),
                selected: false,
            };
            match_vec.push(any);
        }


        let l = get_if(name.clone());
        println!("{}", l);
        //filter to return by names matching "repos" param or else return the last 3 reports.
        let logs_to_report = logs.deref()
                                 .iter()
                                 .filter(|report| match name.to_owned() {
            Select::Repo(name) => report.repo_name == name,
            Select::Any => true,
        })
                                 .take(u)
                                 .collect::<Vec<&Report>>();

        let nv = match_vec.clone();

        //MapBuilders can be inserted with any data type that implements Rustc's Encodable trait
        let material = MapBuilder::new()
            .insert("reports", &logs_to_report.to_owned()).unwrap_or_default()
            .insert("repo_sel", &match_vec.to_owned()).unwrap_or_default()

            .insert("num_sel", &num.to_string()).unwrap_or_default()
            // .insert("count", &counter.to_owned()).unwrap()
            .build();

        //interpolate template data
        tmpl.render_data(&mut canvas, &material).expect(
            "Failed to render \
             data!",
        );

        let mut resp = Response::with((status::Ok, canvas));

        //used hyper here
        resp.headers.set(ContentType(
            Mime(TopLevel::Text, SubLevel::Html, Vec::new()),
        ));
        Ok(resp)
    }
}
