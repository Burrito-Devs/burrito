use std::collections::HashMap;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde_derive::{Serialize, Deserialize};

use super::{systems::{SystemContext, SystemMap}, burrito_cfg::BurritoCfg, burrito_data::BurritoData, log_reader::LogReader, bloom_filter::BloomFilter};

//const TIMESTAMP_REGEX: &str = r#"\[\s[0-9]{4}\.[0-9]{2}\.[0-9]{2}\s[0-9]{2}:[0-9]{2}:[0-9]{2}\s\]"#;
const CHAT_LOG_REGEX: &str = r#"(?<ts>\[ [0-9]{4}\.[0-9]{2}\.[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2} \]) (?<sender>.{1,}) > (?<content>.{1,})"#;
const GAME_LOG_REGEX: &str = r#"(?<ts>\[ [0-9]{4}\.[0-9]{2}\.[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2} \]) \((?<type>[a-z]{1,})\) (?<content>.{1,})"#;
//const TS_FMT: &str = "[ %Y.%m.%d %H:%M:%S ]";
const SYSTEM_MESSAGE_SENDER: &str = "EVE System";
const CHAT_CONNECTION_LOST_MESSAGE: &str = "Connection to chat server lost";
const CHAT_CONNECTION_RESTORED_MESSAGE: &str = "Reconnected to chat server";

pub struct LogWatcher {
    ctx: SystemContext,
    cfg: BurritoCfg,
    data: BurritoData,
    log_readers: Vec<LogReader>,
    old_log_hashes: BloomFilter,
    recent_post_cache: HashMap<(String, String), i64>,
    sys_map: SystemMap,// TODO: should be &SystemMap
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub character_name: String,
    pub event_type: EventType,
    pub trigger: String,
    pub message: String,
}

impl LogWatcher {
    pub fn new(
        ctx: SystemContext,
        cfg: BurritoCfg,
        data: BurritoData,
        sys_map: SystemMap,
    ) -> Self {
        Self {
            ctx,
            cfg,
            data,
            log_readers: vec![],
            old_log_hashes: BloomFilter::new(),
            recent_post_cache: HashMap::new(),
            sys_map,
        }
    }

    pub fn init(&mut self) {
        // Ignore all files that exist before Burrito starts
        self.ignore_logs();
    }

    pub fn get_events(&mut self) -> Vec<LogEvent> {// TODO: Fix game log cooldown logic
        let new_log_readers = self.update_log_readers();
        self.log_readers.extend(new_log_readers);
        let mut events = LogEventQueue::new(self.cfg.game_log_alert_cd_ms);
        let event_time = chrono::offset::Utc::now();
        self.update_recent_post_cache(event_time.timestamp_millis());
        for reader in &mut self.log_readers {
            let result = reader.read_to_end();
            for line in result.lines {
                // TODO: eve time is out of sync with Rust time by like half a minute
                /*let ts_regex = Regex::new(TIMESTAMP_REGEX).unwrap();
                if let Some(ts) = ts_regex.captures(&line) {
                    let ts = ts.get(0).unwrap().as_str();
                    let _ts_result = Utc.datetime_from_str(ts, TS_FMT);
                    event_time = ts_result.unwrap().into();
                }*/
                if reader.is_chatlog_reader() {
                    let regex = Regex::new(CHAT_LOG_REGEX).unwrap();
                    if let Some(cap) = regex.captures(&line) {
                        let sender = &cap["sender"];
                        let content = &cap["content"];
                        let cache_key = (sender.to_owned(), content.to_owned());
                        if self.recent_post_cache.contains_key(&cache_key) {
                            continue;
                        }
                        else {
                            self.recent_post_cache.insert(cache_key, event_time.timestamp_millis());
                        }
                        let d = self.ctx.process_message(content.to_owned(), &self.sys_map);
                        match sender {
                            SYSTEM_MESSAGE_SENDER => {
                                match content {
                                    CHAT_CONNECTION_LOST_MESSAGE => {
                                        events.push_chat_log_event(
                                            LogEvent {
                                                time: event_time,
                                                character_name: reader.get_character_name(),
                                                event_type: EventType::ChatConnectionLost,
                                                trigger: line.to_owned(),
                                                message: CHAT_CONNECTION_LOST_MESSAGE.to_owned(),
                                            }
                                        );
                                    }
                                    CHAT_CONNECTION_RESTORED_MESSAGE => {
                                        events.push_chat_log_event(
                                            LogEvent {
                                                time: event_time,
                                                character_name: reader.get_character_name(),
                                                event_type: EventType::ChatConnectionRestored,
                                                trigger: line.to_owned(),
                                                message: CHAT_CONNECTION_RESTORED_MESSAGE.to_owned(),
                                            }
                                        );
                                    }
                                    _ => {
                                        // TODO: SystemChangedMessage?
                                    }
                                }
                            }
                            _ => {
                                let mut event_type = EventType::NeutInRange(d);
                                let mut message = format!("Hostiles {} jumps away!", d);
                                let content_lower = content.to_lowercase().replace("?", "").replace(".", "");
                                if content_lower.ends_with("status") || content_lower.ends_with("stat") {
                                    event_type = EventType::SystemStatusRequest(d);
                                    message = format!("Status request!");
                                }
                                if content_lower.ends_with("clr") || content_lower.ends_with("clear") {
                                    event_type = EventType::SystemClear(d);
                                    message = format!("System clear!");
                                }
                                events.push_chat_log_event(
                                    LogEvent {
                                        time: event_time,
                                        character_name: reader.get_character_name(),
                                        event_type: event_type,
                                        trigger: line.to_owned(),
                                        message: message
                                    }
                                )
                            }
                        }
                    }
                }
                else {
                    let regex = Regex::new(GAME_LOG_REGEX).unwrap();
                    if let Some(cap) = regex.captures(&line) {
                        let msg_type = &cap["type"];
                        let content = &cap["content"];
                        if msg_type.to_lowercase() == "combat" {// TODO: rewrite as match for other cases
                            for officer_name in self.data.officer_npc_alerts.to_owned() {
                                if content.contains(&officer_name) {
                                    events.push_game_log_event(
                                        LogEvent {
                                            time: event_time,
                                            character_name: reader.get_character_name(),
                                            event_type: EventType::OfficerSpawn,
                                            trigger: line.to_owned(),
                                            message: format!("{} spawn!", officer_name).to_owned(),
                                        }
                                    );
                                }
                            }
                            for special_name in self.data.special_npc_alerts.to_owned() {
                                if content.contains(&special_name) {
                                    events.push_game_log_event(
                                        LogEvent {
                                            time: event_time,
                                            character_name: reader.get_character_name(),
                                            event_type: EventType::DreadSpawn,
                                            trigger: line.to_owned(),
                                            message: format!("{} spawn!", special_name).to_owned(),
                                        }
                                    );
                                }
                            }
                            for faction_string in self.data.faction_npc_alerts.to_owned() {
                                if content.contains(&faction_string) {
                                    events.push_game_log_event(
                                        LogEvent {
                                            time: event_time,
                                            character_name: reader.get_character_name(),
                                            event_type: EventType::FactionSpawn,
                                            trigger: line.to_owned(),
                                            message: format!("{} spawn!", faction_string).to_owned(),
                                        }
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        events.get_log_events().into_iter().cloned().collect()
    }

    fn update_recent_post_cache(&mut self, current_time_ms: i64) {
        let map = self.recent_post_cache.clone();
        let keys = map.keys();
        for key in keys {
            let then = *self.recent_post_cache.get(&key).unwrap();
            if (current_time_ms - then) >= self.cfg.recent_post_cache_ttl_ms {
                self.recent_post_cache.remove(&key);
            }
        }
    }

    fn update_log_readers(&mut self) -> Vec<LogReader> {
        let mut readers = vec![];
        let mut game_log_dir = self.cfg.log_dir.to_owned();
        let mut chat_log_dir = game_log_dir.clone();
        game_log_dir.push_str("/Gamelogs/");
        chat_log_dir.push_str("/Chatlogs/");
        let files = std::fs::read_dir(&game_log_dir)
            .expect("Game log directory not found!");
        files.into_iter().for_each(|file| {
            let file = file.unwrap();
            let filename = file.file_name();
            let filename = filename.to_string_lossy();
            if filename.ends_with(".txt") {
                if !self.old_log_hashes.probably_contains(&filename) {
                    self.old_log_hashes.insert(&filename);
                    let mut file_path = game_log_dir.clone();
                    file_path.push_str(&filename);
                    let mut game_log_reader =
                        LogReader::new_gamelog_reader(&file_path);
                    _ = game_log_reader.read_to_end();
                    readers.push(game_log_reader);
                }
            }
        });
        let files = std::fs::read_dir(&chat_log_dir)
            .expect("Chat log directory not found!");
        files.into_iter().for_each(|file| {
            let file = file.unwrap();
            let filename = file.file_name();
            let filename = filename.to_str().unwrap();
            for channel in self.cfg.text_channel_config.text_channels.iter() {
                if filename.starts_with(channel.get_channel().as_str()) && filename.ends_with(".txt") {
                    if !self.old_log_hashes.probably_contains(&filename) {
                        self.old_log_hashes.insert(&filename);
                        let mut file_path = chat_log_dir.clone();
                        file_path.push_str(&filename);
                        let mut chat_log_reader =
                            LogReader::new_chatlog_reader(&file_path);
                        _ = chat_log_reader.read_to_end();
                        readers.push(chat_log_reader);
                    }
                }
            }
        });
        readers
    }

    // TODO: open most recently modified of each subscribed channel on startup
    fn ignore_logs(&mut self) {
        let mut game_log_dir = self.cfg.log_dir.to_owned();
        let mut chat_log_dir = game_log_dir.clone();
        game_log_dir.push_str("/Gamelogs/");
        chat_log_dir.push_str("/Chatlogs/");
        let files = std::fs::read_dir(&game_log_dir)
            .expect("Game log directory not found!");
        files.into_iter().for_each(|file| {
            let file = file.unwrap();
            let filename = file.file_name();
            let filename = filename.to_string_lossy();
            self.old_log_hashes.insert(&filename);
        });
        let files = std::fs::read_dir(&chat_log_dir)
            .expect("Chat log directory not found!");
        files.into_iter().for_each(|file| {
            let file = file.unwrap();
            let filename = file.file_name();
            let filename = filename.to_str().unwrap();
            self.old_log_hashes.insert(&filename);
        });
    }

}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum EventType {
    NeutInRange(u32),
    SystemClear(u32),
    SystemStatusRequest(u32),
    FactionSpawn,
    DreadSpawn,
    TitanSpawn,
    OfficerSpawn,
    SystemChangedMessage,
    ChatConnectionLost,
    ChatConnectionRestored,
}

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum MessageType {
    ChatMessage,
    GameLogMessage,
}

// TODO: ChatMessageType { player message, channel change, beacon unreachable, etc.  }

#[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum GameMessageType {// TODO: make more comprehensive list of these
    Message,
    DamageIncoming,
    DamageOutgoing,
    NeutIncoming,
    NeutOutgoing,
}

#[derive(Clone, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogEventQueue {
    log_event_cd_ms: u64,
    last_log_event_ms: u64,
    log_events: Vec<LogEvent>,
}

impl LogEventQueue {
    pub fn new(log_event_cd_ms: u64) -> Self {
        Self {
            log_event_cd_ms,
            last_log_event_ms: 0u64,
            log_events: vec![],
        }
    }
    pub fn push_game_log_event(&mut self, log_event: LogEvent) {
        let now = Utc::now().timestamp_millis() as u64;
        if now - self.last_log_event_ms < self.log_event_cd_ms {
            return;
        }
        self.last_log_event_ms = now;
        self.push_chat_log_event(log_event);
    }
    pub fn push_chat_log_event(&mut self, log_event: LogEvent) {
        self.log_events.push(log_event);
    }
    pub fn get_log_events(&self) -> &Vec<LogEvent> {
        &self.log_events
    }
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
