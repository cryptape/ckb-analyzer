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
use chrono::{DateTime, Local, Utc};
use crossbeam::channel::Sender;
use influxdb::{Timestamp, WriteQuery};
use logwatcher::{LogWatcher, LogWatcherAction};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TailLogConfig {
    pub filepath: String,
    pub classify: HashMap<String, String>, // #{ name => regex }
}

pub struct TailLog {
    classify: HashMap<String, Regex>, // #{ name => regex }
    log_watcher: LogWatcher,
    query_sender: Sender<WriteQuery>,
}

impl TailLog {
    pub fn new(config: TailLogConfig, query_sender: Sender<WriteQuery>) -> Self {
        let filepath = PathBuf::from(&config.filepath);
        let classify = config
            .classify
            .into_iter()
            .map(|(category, regex)| {
                (
                    category,
                    Regex::new(&regex).unwrap_or_else(|err| {
                        panic!("invalid regex, str: \"{}\", err: {}", regex, err)
                    }),
                )
            })
            .collect();
        let log_watcher = loop {
            match LogWatcher::register(&filepath) {
                Ok(log_watcher) => break log_watcher,
                Err(_err) => {
                    println!(
                        "[TailLog] failed to open \"{}\", retry in 5s",
                        filepath.to_string_lossy()
                    );
                    sleep(Duration::from_secs(1));
                }
            }
        };
        Self {
            classify,
            log_watcher,
            query_sender,
        }
    }

    pub fn run(&mut self) {
        println!("{} started ...", ::std::any::type_name::<Self>());
        let classify = self.classify.clone();
        let query_sender = self.query_sender.clone();
        self.log_watcher.watch(&mut move |line: String| {
            for (category, regex) in classify.iter() {
                if regex.is_match(line.as_str()) {
                    let query = measurement::Log {
                        time: log_time(&line),
                        marker: 1,
                        category: category.clone(),
                        raw: line.clone(),
                    }
                    .into_write_query();
                    query_sender.send(query).unwrap();
                }
            }
            LogWatcherAction::None
        });
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
            eprintln!("failed to parse datetime, use local time");
            Utc::now().into()
        }
    }
}
