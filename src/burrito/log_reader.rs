use std::collections::BTreeSet;
use std::{fs, fs::File, io::Read};
use std::io::{BufReader, SeekFrom, Seek};

use encoding_rs::{UTF_16LE, UTF_8};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use super::burrito_cfg::BurritoCfg;

const LISTENER_REGEX: &str = r#"\s{1,}Listener:\s{1,}(?<listener>[A-z ]{1,})"#;

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogReader {
    character_name: String,
    log_file: String,
    cursor: usize,
    is_chatlog_reader: bool,
}

impl LogReader {
    pub fn new_intel_reader(config: BurritoCfg, intel_channel: IntelChannel) -> Self {
        let mut log_reader =
            Self {
                character_name: String::new(),
                log_file: get_latest_chat_log_file(&config.log_dir, &intel_channel),
                cursor: 0usize,
                is_chatlog_reader: true,
            };
        extract_listener(&mut log_reader);
        log_reader
    }
    pub fn new_game_log_readers(config: BurritoCfg, count: u32) -> Vec<Self> {
        let mut log_readers = vec![];
        let log_files = get_latest_game_log_files(&config.log_dir, count);
        for log_file in log_files {
            let mut log_reader =
                Self {
                    character_name: String::new(),
                    log_file: log_file,
                    cursor: 0usize,
                    is_chatlog_reader: false,
                };
            extract_listener(&mut log_reader);
            log_readers.push(log_reader);
        }
        log_readers
    }
    pub fn read_to_end(&mut self) -> LogReadResult {
        let mut lines: Vec<String> = vec![];
        let f = File::open(&self.log_file).unwrap();
        let mut reader = BufReader::new(f);
        _ = reader.seek(SeekFrom::Start(self.cursor as u64));
        let mut buffer = vec![];
        let read = reader.read_to_end(&mut buffer).unwrap();
        if read > 0 {
            let (data, _, _) = if self.is_chatlog_reader() { UTF_16LE.decode(&buffer) } else { UTF_8.decode(&buffer) };
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

fn get_latest_chat_log_file(log_dir: &str, channel: &IntelChannel) -> String {
    let mut log_dir = log_dir.to_owned();
    log_dir.push_str("/Chatlogs/");
    let mut sorted_files: BTreeSet<String> = BTreeSet::new();
    let files = fs::read_dir(&log_dir).unwrap();
    for file in files {
        let file_name = file.unwrap().file_name();
        let file_name = file_name.to_str().unwrap();
        if file_name.starts_with(channel.get_channel().as_str()) {
            sorted_files.insert(log_dir.to_owned() + file_name);
        }
    }
    return sorted_files.last().unwrap().to_owned()
}

fn get_latest_game_log_files(log_dir: &str, count: u32) -> Vec<String> {
    let mut log_dir = log_dir.to_owned();
    log_dir.push_str("/Gamelogs/");
    let mut sorted_files: BTreeSet<String> = BTreeSet::new();
    let files = fs::read_dir(&log_dir).unwrap();
    for file in files {
        let file_name = file.unwrap().file_name();
        let file_name = file_name.to_str().unwrap();
        sorted_files.insert(log_dir.to_owned() + file_name);
    }
    return sorted_files.iter().rev().take(count as usize).cloned().collect()
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

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum IntelChannel {
    Aridia,
    Branch,
    Catch,
    CloudRing,
    CobaltEdge,
    Curse,
    Deklein,
    Delve,
    Fade,
    Fountain,
    Geminate,
    Khanid,
    Lonetrek,
    ParagonSoul,
    PeriodBasis,
    Pochven,
    Providence,
    PureBlind,
    Querious,
    Syndicate,
    Tenal,
    Tribute,
    ValeOfTheSilent,
    Venal,
    Gj,
    Custom{channel: String}
}

impl IntelChannel {
    fn get_channel(&self) -> String {
        match self {
            IntelChannel::Aridia => "aridia.imperium".to_owned(),
            IntelChannel::Branch => "brn.imperium".to_owned(),
            IntelChannel::Catch => "catch.imperium".to_owned(),
            IntelChannel::CloudRing => "cr.imperium".to_owned(),
            IntelChannel::CobaltEdge => "ce.imperium".to_owned(),
            IntelChannel::Curse => "curse.imperium".to_owned(),
            IntelChannel::Deklein => "dek.imperium".to_owned(),
            IntelChannel::Delve => "delve.imperium".to_owned(),
            IntelChannel::Fade => "fade.imperium".to_owned(),
            IntelChannel::Fountain => "ftn.imperium".to_owned(),
            IntelChannel::Geminate => "gem.imperium".to_owned(),
            IntelChannel::Khanid => "khanid.imperium".to_owned(),
            IntelChannel::Lonetrek => "lone.imperium".to_owned(),
            IntelChannel::ParagonSoul => "paragon.imperium".to_owned(),
            IntelChannel::PeriodBasis => "period.imperium".to_owned(),
            IntelChannel::Pochven => "triangle.imperium".to_owned(),
            IntelChannel::Providence => "provi.imperium".to_owned(),
            IntelChannel::PureBlind => "pb.imperium".to_owned(),
            IntelChannel::Querious => "querious.imperium".to_owned(),
            IntelChannel::Syndicate => "synd.imperium".to_owned(),
            IntelChannel::Tenal => "tnl.imperium".to_owned(),
            IntelChannel::Tribute => "tri.imperium".to_owned(),
            IntelChannel::ValeOfTheSilent => "vale.imperium".to_owned(),
            IntelChannel::Venal => "vnl.imperium".to_owned(),
            IntelChannel::Gj => "gj.imperium".to_owned(),
            Self::Custom { channel } => channel.to_owned(),
        }
    }
}