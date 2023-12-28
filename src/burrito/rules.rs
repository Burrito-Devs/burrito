use serde_derive::{Deserialize, Serialize};

pub trait Comparable: Eq + Ord + PartialEq + PartialOrd {}
impl<T: Eq + Ord + PartialEq + PartialOrd> Comparable for T {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EventRule {
    rule: ComparisonRule,
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
        use crate::burrito::log_event::{EventType, LogEventMetadata, LogEvent};

        let mut uut = LogEvent {
            time: Utc::now(),
            character_name: "fgrtgdfhfdgh".to_owned(),
            event_type: EventType::RangeOfSystem,
            trigger: "qwer".to_owned(),
            message: "asdflkasdflaksdf".to_owned(),
            event_metadata: LogEventMetadata::default(),
        };

        uut.event_metadata.put("distance", 5u32);
        assert_eq!(Some(5u32), uut.event_metadata.get("distance"));
        let asdf: u32 = uut.event_metadata.get("distance").unwrap();
        assert_eq!(5u32, asdf);

    }

}
