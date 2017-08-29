//! database Module
//!
//! This module houses simple DB functionality and implementations.

use rusqlite::SqliteConnection;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc;
use std::thread;
use std::io;
use std::fs;
use std::env;
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use reports::{Report, Reports};
use timestamp::Timestamp;


pub struct Record {
    reports: Vec<u8>
}