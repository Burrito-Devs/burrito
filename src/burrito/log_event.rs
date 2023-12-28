use std::{borrow::Borrow, str::FromStr};
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub character_name: String,
    pub event_type: EventType,
    pub trigger: String,
    pub message: String,
    pub event_metadata: LogEventMetadata,
}

impl LogEvent {
    fn add_metadata(&mut self, k: impl ToString, v: impl ToString) {
        self.event_metadata.put(k, v);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum EventType {
    RangeOfCharacter,
    RangeOfSystem,
    SystemClear,
    SystemStatusRequest,
    ChatlogMessage,
    GamelogMessage,
    FactionSpawn,
    DreadSpawn,
    TitanSpawn,
    OfficerSpawn,
    SystemChangedMessage,
    ChatConnectionLost,
    ChatConnectionRestored,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LogEventMetadata {
    metadata: HashMap<String, String>,
}

impl LogEventMetadata {
    pub fn new() -> Self {
        return Default::default();
    }
    pub fn get<K: Borrow<str>, T: FromStr>(&self, key: K) -> Option<T> {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return value.as_str().parse().ok();
        }
        None
    }
    pub fn put(&mut self, key: impl ToString, value: impl ToString) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

mod test {
    #[test]
    fn test_test_test() {
        use crate::burrito::log_event::LogEventMetadata;

        let mut uut = LogEventMetadata::new();
        assert_eq!(None, uut.get::<&str, String>("does not exist"));
        uut.put("u32", 5u32);
        uut.put("f64".to_string(), 0.25f64);
        uut.put("String", "My String");
        assert_eq!(None, uut.get::<String, i64>("does not exist".to_string()));
        assert_eq!(Some(5u32), uut.get("u32"));
        assert_eq!(Some(0.25f64), uut.get("f64"));
        assert_eq!(Some("My String".to_string()), uut.get("String"));
    }
}
