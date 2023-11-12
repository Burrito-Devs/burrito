use std::{fs::File, io::Read};
use std::io::{BufReader, SeekFrom, Seek};

use encoding_rs::{UTF_16LE, UTF_8};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

const LISTENER_REGEX: &str = r#"\s{1,}Listener:\s{1,}(?<listener>[A-z ]{1,})"#;
const CHANNEL_REGEX: &str = r#"\s{1,}Channel Name:\s{1,}(?<channel>.{1,})"#;

// TODO: LogReader should probably be a trait and chat and game log readers should be different implementations
#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogReader {
    character_name: String,
    channel_name: String,
    log_file: String,
    cursor: usize,
    is_chatlog_reader: bool,
}

impl LogReader {

    pub fn new_chatlog_reader(file: &str) -> Self {
        let mut log_reader =
            Self {
                character_name: String::new(),
                channel_name: String::new(),
                log_file: file.to_owned(),
                cursor: 0usize,
                is_chatlog_reader: true,
            };
        extract_channel_info(&mut log_reader);
        log_reader
    }

    pub fn new_gamelog_reader(file: &str) -> Self {
        let mut log_reader =
            Self {
                character_name: String::new(),
                channel_name: String::new(),
                log_file: file.to_owned(),
                cursor: 0usize,
                is_chatlog_reader: false,
            };
        extract_channel_info(&mut log_reader);
        log_reader
    }

    // TODO: refactor this. Make read function extract listener/channel if still blank
    pub fn read_to_end(&mut self) -> LogReadResult {
        if self.character_name.len() == 0 || self.channel_name.len() == 0 {
            extract_channel_info(self);
        }
        let mut lines: Vec<String> = vec![];
        let f = File::open(&self.log_file).expect(&format!("Failed to open {}", self.log_file));
        let mut reader = BufReader::new(f);
        _ = reader.seek(SeekFrom::Start(self.cursor as u64));
        let mut buffer = vec![];
        let read = reader.read_to_end(&mut buffer).unwrap();
        if read > 0 {
            let (data, _, _) = if self.is_chatlog_reader() {
                UTF_16LE.decode(&buffer)
            }
            else {
                UTF_8.decode(&buffer)
            };
            self.cursor += read;
            for line in data.trim().split("\r\n") {
                lines.push(line.to_string());
            }
        }
        return LogReadResult { bytes_read: read, lines: lines };
    }

    pub fn is_chatlog_reader(&self) -> bool {
        self.is_chatlog_reader
    }

    pub fn get_character_name(&self) -> String {
        self.character_name.to_owned()
    }

    pub fn get_log_file(&self) -> String {
        self.log_file.to_owned()
    }

}

fn extract_channel_info(log_reader: &mut LogReader) {
    let listener_regex = Regex::new(LISTENER_REGEX).unwrap();
    let channel_regex = Regex::new(CHANNEL_REGEX).unwrap();
    let mut lines: Vec<String> = vec![];
    let f = File::open(&log_reader.log_file).expect(&format!("Failed to open {}", log_reader.log_file));
    let mut buf_reader = BufReader::new(f);
    let mut buffer = vec![];
    let read = buf_reader.read_to_end(&mut buffer).unwrap();
    if read > 0 {
        let (data, _, _) = if log_reader.is_chatlog_reader() {
            UTF_16LE.decode(&buffer)
        }
        else {
            UTF_8.decode(&buffer)
        };
        for line in data.trim().split("\r\n") {
            lines.push(line.to_string());
        }
    }
    let result = LogReadResult { bytes_read: read, lines: lines };
    for line in result.lines {
        if let Some(cap) = listener_regex.captures(&line) {
            log_reader.character_name = cap["listener"].to_owned();
        }
        if let Some(cap) = channel_regex.captures(&line) {
            log_reader.channel_name = cap["channel"].to_owned();
        }
    }
}

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogReadResult {
    pub bytes_read: usize,
    pub lines: Vec<String>,
}
