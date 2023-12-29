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
    fn add_metadata(&mut self, k: impl ToString, v: impl ToString, t: LogEventMetadataType) {
        self.event_metadata.put(k, v, t);
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
    metadata: HashMap<String, LogEventMetadataField>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum LogEventMetadataType {
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogEventMetadataField {
    data: String,
    data_type: LogEventMetadataType,
}

impl LogEventMetadataField {
    pub fn new(data: impl ToString, data_type: LogEventMetadataType) -> Self {
        LogEventMetadataField {
            data: data.to_string(),
            data_type,
        }
    }
    pub fn get_data(&self) -> &String {
        return &self.data;
    }
    pub fn get_data_type(&self) -> LogEventMetadataType {
        return self.data_type;
    }
}

impl LogEventMetadata {
    pub fn new() -> Self {
        return Default::default();
    }
    pub fn get<K: Borrow<str>, T: FromStr>(&self, key: K) -> Option<T> {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return value.get_data().as_str().parse().ok();
        }
        None
    }
    pub fn put(&mut self, key: impl ToString, value: impl ToString, data_type: LogEventMetadataType) {
        self.metadata.insert(key.to_string(), LogEventMetadataField::new(value.to_string(), data_type));
    }
}

mod test {

    #[test]
    fn test_test_test() {
        use crate::burrito::log_event::{LogEventMetadata, LogEventMetadataType};

        let mut uut = LogEventMetadata::new();
        assert_eq!(None, uut.get::<&str, String>("does not exist"));
        uut.put("u32", 5u32, LogEventMetadataType::U32);
        uut.put("f64".to_string(), 0.25f64, LogEventMetadataType::F64);
        uut.put("String", "My String", LogEventMetadataType::String);
        assert_eq!(None, uut.get::<String, i64>("does not exist".to_string()));
        assert_eq!(Some(5u32), uut.get("u32"));
        assert_eq!(Some(0.25f64), uut.get("f64"));
        assert_eq!(Some("My String".to_string()), uut.get("String"));
    }
}
