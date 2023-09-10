use std::{fs::File, io::Read};
use std::io::{BufReader, SeekFrom, Seek};

use encoding_rs::{UTF_16LE, UTF_8};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

const LISTENER_REGEX: &str = r#"\s{1,}Listener:\s{1,}(?<listener>[A-z ]{1,})"#;

// TODO: LogReader should probably be a trait and chat and game log readers should be different implementations
#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogReader {
    character_name: String,
    log_file: String,
    cursor: usize,
    is_chatlog_reader: bool,
}

impl LogReader {
    pub fn new_chatlog_reader(file: &str) -> Self {
        let mut log_reader =
            Self {
                character_name: String::new(),
                log_file: file.to_owned(),
                cursor: 0usize,
                is_chatlog_reader: true,
            };
        extract_listener(&mut log_reader);
        log_reader
    }
    pub fn new_gamelog_reader(file: &str) -> Self {
        let mut log_reader =
            Self {
                character_name: String::new(),
                log_file: file.to_owned(),
                cursor: 0usize,
                is_chatlog_reader: false,
            };
        extract_listener(&mut log_reader);
        log_reader
    }
    pub fn read_to_end(&mut self) -> LogReadResult {
        let mut lines: Vec<String> = vec![];
        let f = File::open(&self.log_file).unwrap();
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
}

fn extract_listener(log_reader: &mut LogReader) {
    let regex = Regex::new(LISTENER_REGEX).unwrap();
    let result = log_reader.read_to_end();
    for line in result.lines {
        if let Some(cap) = regex.captures(&line) {
            log_reader.character_name = cap["listener"].to_owned();
            break;
        }
    }
}

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogReadResult {
    pub bytes_read: usize,
    pub lines: Vec<String>,
}
