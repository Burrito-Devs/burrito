use chrono::{DateTime, Utc, TimeZone};
use regex::Regex;
use serde_derive::{Serialize, Deserialize};

use super::{systems::{SystemContext, SystemMap}, burrito_cfg::BurritoCfg, burrito_data::BurritoData, log_reader::LogReader};

const TIMESTAMP_REGEX: &str = r#"\[\s[0-9]{4}\.[0-9]{2}\.[0-9]{2}\s[0-9]{2}:[0-9]{2}:[0-9]{2}\s\]"#;
const CHAT_LOG_REGEX: &str = r#"(?<ts>\[ [0-9]{4}\.[0-9]{2}\.[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2} \]) (?<sender>.{1,}) > (?<content>.{1,})"#;
const GAME_LOG_REGEX: &str = r#"(?<ts>\[ [0-9]{4}\.[0-9]{2}\.[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2} \]) \((?<type>[a-z]{1,})\) (?<content>.{1,})"#;
const TS_FMT: &str = "[ %Y.%m.%d %H:%M:%S ]";
const SYSTEM_MESSAGE_SENDER: &str = "EVE System";
const CHAT_CONNECTION_LOST_MESSAGE: &str = "Connection to chat server lost";
const CHAT_CONNECTION_RESTORED_MESSAGE: &str = "Reconnected to chat server";

pub struct LogWatcher {
    ctx: SystemContext,
    cfg: BurritoCfg,
    data: BurritoData,
    log_readers: Vec<LogReader>,
    event_types: Vec<EventType>,
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
        log_readers: Vec<LogReader>,
        event_types: Vec<EventType>,
        sys_map: SystemMap,
    ) -> Self {
        Self {
            ctx,
            cfg,
            data,
            log_readers,
            event_types,
            sys_map,
        }
    }

    pub fn get_all_events(&mut self) -> Vec<LogEvent> {// TODO: Fix game log cooldown logic
        let mut events = LogEventQueue::new(self.cfg.game_log_alert_cd_ms);
        self.log_readers.iter_mut().for_each( |reader| {
            let result = reader.read_to_end();
            for line in result.lines {
                let mut event_time = chrono::offset::Utc::now();
                let ts_regex = Regex::new(TIMESTAMP_REGEX).unwrap();
                if let Some(ts) = ts_regex.captures(&line) {
                    let ts = ts.get(0).unwrap().as_str();
                    let ts_result = Utc.datetime_from_str(ts, TS_FMT);
                    event_time = ts_result.unwrap().into();
                }
                if reader.is_chatlog_reader() {
                    let regex = Regex::new(CHAT_LOG_REGEX).unwrap();
                    if let Some(cap) = regex.captures(&line) {
                        let sender = &cap["sender"];
                        let content = &cap["content"];
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
        });
        events.get_log_events().into_iter().cloned().collect()
    }

    pub fn get_events(&mut self) -> Vec<LogEvent> {
        self.get_all_events().into_iter()
            .filter(|event| self.event_types.contains(&event.event_type)).collect()
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
