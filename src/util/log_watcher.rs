use std::fs::Metadata;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader, SeekFrom};
use tokio::time::{delay_for, Duration};

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;

/// `LogWatcher` is a Rust Stream which behaviours like `tail`.
///
/// `LogWatcher` accepts parameter `seek: enum { SeekToStart, SeekToEnd }`,
/// to support the start position. By default `seek = SeekToEnd`.
///
/// On polling, `LogWatcher` reopen if the file rotated when trying
/// to read line at the last recorded position; otherwise returns the
/// new line.
pub struct LogWatcher {
    filepath: String,
    // file: File,
    reader: BufReader<File>,
    metadata: Metadata,
    // pos: u64,
}

impl LogWatcher {
    pub async fn new(filepath: impl AsRef<Path>) -> Self {
        let filepath = filepath.as_ref().to_string_lossy().to_string();
        let (mut file, metadata) = Self::open(&filepath).await;
        let pos = metadata.len();
        file.seek(SeekFrom::Start(pos)).await.unwrap();
        let reader = BufReader::new(file);
        Self {
            filepath,
            reader,
            metadata,
        }
    }

    pub async fn watch<Callback: ?Sized>(&mut self, callback: &mut Callback)
    where
        Callback: FnMut(String),
    {
        let mut line = String::new();
        loop {
            line.clear();
            match self.reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF, try reopen
                    self.reopen_if_log_rotated().await;
                }

                Ok(_num_bytes) => {
                    callback(line.replace("\n", ""));

                    // self.pos += num_bytes as u64;
                    // self.file.seek(SeekFrom::Start(self.pos));
                }

                Err(err) => log::error!("read {}, error: {}", self.filepath, err),
            }
        }
    }

    async fn reopen_if_log_rotated(&mut self) {
        let (file, metadata) = Self::open(&self.filepath).await;
        if metadata.st_ino() != self.metadata.st_ino() {
            self.reader = BufReader::new(file);
            self.metadata = metadata;
            // self.pos = self.metadata.len();
            // self.file.seek(SeekFrom::Start(0));
        }
    }

    async fn open(filepath: &str) -> (File, Metadata) {
        loop {
            let file = match File::open(filepath).await {
                Ok(file) => file,
                Err(err) => {
                    log::error!("open {}, error: {}", filepath, err);
                    delay_for(Duration::from_secs(1)).await;
                    continue;
                }
            };
            let metadata = match file.metadata().await {
                Ok(metadata) => metadata,
                Err(err) => {
                    log::error!("read file metadata {}, error: {}", filepath, err);
                    delay_for(Duration::from_secs(1)).await;
                    continue;
                }
            };
            return (file, metadata);
        }
    }
}
