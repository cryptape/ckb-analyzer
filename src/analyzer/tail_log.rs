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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone)]
pub struct Regex(#[serde(with = "serde_regex")] regex::Regex);

impl Deref for Regex {
    type Target = regex::Regex;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::std::fmt::Debug for Regex {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Debug::fmt(&self.0, f)
    }
}

impl ::std::fmt::Display for Regex {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.0, f)
    }
}

pub struct TailLog {
    patterns: HashMap<String, Regex>, // #{ name => regex }
    log_watcher: LogWatcher,
    query_sender: Sender<WriteQuery>,
}

impl TailLog {
    pub fn new<P: AsRef<Path>>(
        filepath: P,
        patterns: HashMap<String, Regex>,
        query_sender: Sender<WriteQuery>,
    ) -> Self {
        let log_watcher = loop {
            match LogWatcher::register(filepath.as_ref()) {
                Ok(log_watcher) => break log_watcher,
                Err(_err) => {
                    log::warn!(
                        "[TailLog] failed to open \"{}\", retry in 5s",
                        filepath.as_ref().to_string_lossy().to_string(),
                    );
                    sleep(Duration::from_secs(1));
                }
            }
        };
        Self {
            patterns,
            log_watcher,
            query_sender,
        }
    }

    pub fn run(&mut self) {
        let patterns = self.patterns.clone();
        let query_sender = self.query_sender.clone();
        self.log_watcher.watch(&mut move |line: String| {
            for (category, regex) in patterns.iter() {
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
            log::error!("failed to parse datetime, use local time");
            Utc::now().into()
        }
    }
}
