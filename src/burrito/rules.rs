use std::cmp::Ordering;

use serde_derive::{Deserialize, Serialize};

use super::log_event::{LogEventMetadataField, LogEventMetadataType};

pub trait Comparable: Eq + Ord + PartialEq + PartialOrd {}
impl<T: Eq + Ord + PartialEq + PartialOrd> Comparable for T {}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EventRule {
    metadata_tag: String,
    rule: ComparisonRule,
    ref_value: String,
}

pub fn compare(data_field1: LogEventMetadataField, data_field2: LogEventMetadataField, cmp_type: ComparisonRule) -> bool {
    let ordering = data_field_ord(data_field1, data_field2);
    match cmp_type {
        ComparisonRule::LessThan => ordering == Ordering::Less,
        ComparisonRule::LessThanOrEqualTo => (ordering == Ordering::Less) || (ordering == Ordering::Equal),
        ComparisonRule::NotEqualTo => ordering != Ordering::Equal,
        ComparisonRule::EqualTo => ordering == Ordering::Equal,
        ComparisonRule::GreaterThanOrEqualTo => (ordering == Ordering::Greater) || (ordering == Ordering::Equal),
        ComparisonRule::GreaterThan => ordering == Ordering::Greater,
    }
}

fn data_field_ord(data_field1: LogEventMetadataField, data_field2: LogEventMetadataField) -> Ordering {
    // Types are not equal, use String ordering
    if data_field1.get_data_type() != data_field2.get_data_type() {
        data_field1.get_data().cmp(data_field2.get_data())
    }
    // Types are the same, use typed ordering
    else {
        match data_field1.get_data_type() {
            LogEventMetadataType::U32 => {
                let value1 = data_field1.get_data().parse::<u32>().unwrap();
                let value2 = data_field2.get_data().parse::<u32>().unwrap();
                value1.cmp(&value2)
            },
            LogEventMetadataType::I32 => {
                let value1 = data_field1.get_data().parse::<i32>().unwrap();
                let value2 = data_field2.get_data().parse::<i32>().unwrap();
                value1.cmp(&value2)
            },
            LogEventMetadataType::U64 => {
                let value1 = data_field1.get_data().parse::<u64>().unwrap();
                let value2 = data_field2.get_data().parse::<u64>().unwrap();
                value1.cmp(&value2)
            },
            LogEventMetadataType::I64 => {
                let value1 = data_field1.get_data().parse::<i64>().unwrap();
                let value2 = data_field2.get_data().parse::<i64>().unwrap();
                value1.cmp(&value2)
            },
            LogEventMetadataType::F32 => {
                let value1 = data_field1.get_data().parse::<f32>().unwrap();
                let value2 = data_field2.get_data().parse::<f32>().unwrap();
                ord_f32(value1, value2)
            },
            LogEventMetadataType::F64 => {
                let value1 = data_field1.get_data().parse::<f64>().unwrap();
                let value2 = data_field2.get_data().parse::<f64>().unwrap();
                ord_f64(value1, value2)
            },
            LogEventMetadataType::String => {
                data_field1.cmp(&data_field2)
            },
        }
    }
}

fn ord_f32(v1: f32, v2: f32) -> Ordering {
    if v1 > v2 {
        Ordering::Greater
    }
    else if v2 > v1 {
        Ordering::Less
    }
    else {
        Ordering::Equal
    }
}

fn ord_f64(v1: f64, v2: f64) -> Ordering {
    if v1 > v2 {
        Ordering::Greater
    }
    else if v2 > v1 {
        Ordering::Less
    }
    else {
        Ordering::Equal
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ComparisonRule {
    LessThan,
    LessThanOrEqualTo,
    NotEqualTo,
    EqualTo,
    GreaterThanOrEqualTo,
    GreaterThan,
}

mod test {

    #[test]
    fn test_test_test() {
        use chrono::Utc;    
        use crate::burrito::log_event::{EventType, LogEventMetadata, LogEventMetadataType, LogEvent};

        let mut uut = LogEvent {
            time: Utc::now(),
            character_name: "fgrtgdfhfdgh".to_owned(),
            event_type: EventType::RangeOfSystem,
            trigger: "qwer".to_owned(),
            message: "asdflkasdflaksdf".to_owned(),
            event_metadata: LogEventMetadata::default(),
        };

        uut.event_metadata.put("distance", 5u32, LogEventMetadataType::U32);
        assert_eq!(Some(5u32), uut.event_metadata.get("distance"));
        let asdf: u32 = uut.event_metadata.get("distance").unwrap();
        assert_eq!(5u32, asdf);
    }

}
