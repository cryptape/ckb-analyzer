//! This module monitors the logfile and reports significant event logs.
//!
//! This module produces the below measures:
//!   - [Log](TODO link)
//!
//! ### How it works?
//!
//! We continuously tail the logfile, report the raw lines if the log-level is "ERROR" or
//! "WARNING".
//!
//! ### Why measure it?
//!
//! Logs is the most common way to record abnormal and significant events. And we care about these
//! events which helps us understand the program state, detect abnormal events, and so on.

use crate::measurement::{self, IntoWriteQuery};
use crate::util::LogWatcher;
use chrono::{DateTime, Local, Utc};
use crossbeam::channel::Sender;
use influxdb::{Timestamp, WriteQuery};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) struct PatternLogs {
    filepath: PathBuf,
    patterns: HashMap<String, Regex>,
    query_sender: Sender<WriteQuery>,
}

impl PatternLogs {
    pub(crate) async fn new<P: AsRef<Path>>(data_dir: P, query_sender: Sender<WriteQuery>) -> Self {
        let filepath = data_dir.as_ref().join("logs").join("run.log");
        let patterns = {
            let mut patterns = HashMap::new();
            patterns.insert("error".to_string(), Regex::new(" ERROR ").unwrap());
            patterns
        };
        Self {
            filepath,
            patterns,
            query_sender,
        }
    }

    pub(crate) async fn run(&mut self) {
        let mut log_watcher = LogWatcher::new(&self.filepath).await;
        log_watcher
            .watch(&mut |line: String| {
                for (category, regex) in self.patterns.iter() {
                    if regex.is_match(line.as_str()) {
                        log::info!("line: {}", line);
                        let query = measurement::Log {
                            time: log_time(&line),
                            marker: 1,
                            category: category.clone(),
                            raw: line.clone(),
                        }
                        .into_write_query();
                        self.query_sender.send(query).unwrap();
                    }
                }
            })
            .await;
    }
}

fn log_time(line: &str) -> Timestamp {
    let datetime_length = Local::now()
        .format("%Y-%m-%d %H:%M:%S%.3f %Z")
        .to_string()
        .len();
    match DateTime::parse_from_str(&line[..datetime_length], "%Y-%m-%d %H:%M:%S%.3f %z") {
        Ok(datetime) => datetime.into(),
        Err(_) => {
            log::error!("failed to parse datetime, use local time");
            Utc::now().into()
        }
    }
}
