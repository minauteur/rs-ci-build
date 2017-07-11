// use hooks;
// use logging::HasLogger;

use toml;

use serde_json;

use std::io::Read;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;

//struct for deserializing "dir.toml" from repo root
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
   pub repositories: BTreeMap<String, String>
}

impl Config {
    pub fn new(path: String) -> Config {
        let file = PathBuf::from(path);
        let mut f = match File::open(file) {
            Err(e) => panic!("\ncouldn't open config!\n{}", &e),
            Ok(file) => file,
        };
        let mut buf = String::new();
        match f.read_to_string(&mut buf) {
            Err(e) => panic!("\ncouldn't read!\n{}", &e),
            Ok(buf) => buf,
        };
        let config: Config = match toml::from_str(&buf) {
            Ok(btree) => btree,
            Err(e) => {
                panic!("No repo objects in 'repo_config.toml'!\n{}", &e);
            },
        };
        Config {
            repositories: config.repositories,
        }
    }
}
    // let url_query_map = match req.get::<UrlEncodedQuery>() {
    //     Ok(hashmap) => hashmap,
    //     Err(why) => {
    //         error!(logger, "\nreq.get::UrlEncodedQuery>() \
    //             failed to return UrlQueryMap"; "reason" => %why);
    //         let err_resp = format!("\nCould not extract HashMap from url...\nError: {}\n", &why);
    //         return Err(IronError::new(
    //             CIError::UrlDecoding(UrlDecodingError::EmptyQuery),
    //             (status::BadRequest, err_resp),
    //         ));
    //     },
    // };


    // let path = match url_query_map.get("path").and_then(
    //     |path_vec| path_vec.iter().next(),
    // ) {
    //     Some(path_value) => path_value,
    //     None => {
    //         info!(logger, "\nNo value for 'path' key in UrlQueryMap!\n");
    //         let no_val = format!(
    //             "\nProblem unpacking urlencoded HashMap: No value(s) exist for given \
    //              key!\n"
    //         );
    //         return Err(IronError::new(
    //             CIError::UrlDecoding(UrlDecodingError::EmptyQuery),
    //             (status::BadRequest, no_val),
    //         ));
    //     },
    // };