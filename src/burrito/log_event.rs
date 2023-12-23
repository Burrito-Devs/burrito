use std::borrow::Borrow;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use enum_index_derive::{EnumIndex, IndexEnum};
use serde_derive::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub character_name: String,
    pub event_type: EventType,
    pub trigger: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, EnumIndex, IndexEnum, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
    metadata: HashMap<String, (EventDataType, String)>,
}

impl LogEventMetadata {
    pub fn new() -> Self {
        return Default::default();
    }
    pub fn get_string<K: Borrow<str>>(&self, key: K) -> Option<String> {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::String => return Some(value.1),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_u8<K: Borrow<str>>(&self, key: K) -> Option<u8> {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::U8 => return Some(value.1.parse::<u8>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_i8<K>(&self, key: K) -> Option<i8>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::I8 => return Some(value.1.parse::<i8>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type")
            }
        }
        return None;
    }
    pub fn get_u16<K>(&self, key: K) -> Option<u16>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::U16 => return Some(value.1.parse::<u16>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_i16<K>(&self, key: K) -> Option<i16>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::I16 => return Some(value.1.parse::<i16>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_u32<K>(&self, key: K) -> Option<u32>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::U32 => return Some(value.1.parse::<u32>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_i32<K>(&self, key: K) -> Option<i32>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::I32 => return Some(value.1.parse::<i32>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_u64<K>(&self, key: K) -> Option<u64>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::U64 => return Some(value.1.parse::<u64>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_i64<K>(&self, key: K) -> Option<i64>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::I64 => return Some(value.1.parse::<i64>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_f32<K>(&self, key: K) -> Option<f32>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::F32 => return Some(value.1.parse::<f32>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn get_f64<K>(&self, key: K) -> Option<f64>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            match value.0 {
                EventDataType::F64 => return Some(value.1.parse::<f64>().expect("BUG! Invalid value type")),
                _ => panic!("BUG! Invalid value type"),
            }
        }
        return None;
    }
    pub fn put(&mut self, key: impl ToString, value: impl ToString, value_type: EventDataType) {
        self.metadata.insert(key.to_string(), (value_type, value.to_string()));
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum EventDataType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
}

mod test {
    #[test]
    fn test_test_test() {
        use crate::burrito::log_event::{EventDataType, LogEventMetadata};

        let mut uut = LogEventMetadata::new();
        assert_eq!(None, uut.get_string("does not exist"));
        uut.put("u32", 5u32, EventDataType::U32);
        uut.put("f64".to_string(), 0.25f64, EventDataType::F64);
        uut.put("String", "My String", EventDataType::String);
        assert_eq!(None, uut.get_string("does not exist".to_string()));
        assert_eq!(Some(5u32), uut.get_u32("u32"));
        assert_eq!(Some(0.25f64), uut.get_f64("f64"));
        assert_eq!(Some("My String".to_string()), uut.get_string("String"));
    }
}
