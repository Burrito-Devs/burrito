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

#[derive(Clone, Debug, EnumIndex, IndexEnum, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum EventType {
    RangeOfCharacter(u32),
    RangeOfSystem(u32),
    SystemClear(u32),
    SystemStatusRequest(u32),
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
    pub fn get_string<K>(&self, key: K) -> Option<String>
    where K: Borrow<str>,
    {
        return self.metadata.get(key.borrow()).cloned();
    }
    pub fn get_u8<K>(&self, key: K) -> Option<u8>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<u8>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_i8<K>(&self, key: K) -> Option<i8>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<i8>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_u16<K>(&self, key: K) -> Option<u16>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<u16>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_i16<K>(&self, key: K) -> Option<i16>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<i16>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_u32<K>(&self, key: K) -> Option<u32>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<u32>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_i32<K>(&self, key: K) -> Option<i32>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<i32>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_u64<K>(&self, key: K) -> Option<u64>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<u64>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_i64<K>(&self, key: K) -> Option<i64>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<i64>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_f32<K>(&self, key: K) -> Option<f32>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<f32>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn get_f64<K>(&self, key: K) -> Option<f64>
    where K: Borrow<str>,
    {
        if let Some(value) = self.metadata.get(key.borrow()) {
            return Some(value.parse::<f64>().expect("BUG! invalid value type"));
        }
        return None;
    }
    pub fn put(&mut self, key: impl ToString, value: impl ToString) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

mod test {
    #[test]
    fn test_test_test() {
        let mut uut = crate::burrito::log_event::LogEventMetadata::new();
        assert_eq!(None, uut.get_string("does not exist"));
        uut.put("u32", 5u32);
        uut.put("f64".to_string(), 0.25f64);
        uut.put("String", "My String");
        assert_eq!(None, uut.get_string("does not exist".to_string()));
        assert_eq!(Some(5u32), uut.get_u32("u32"));
        assert_eq!(Some(0.25f64), uut.get_f64("f64"));
        assert_eq!(Some("My String".to_string()), uut.get_string("String"));
    }
}
