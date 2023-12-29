use std::collections::HashSet;

use super::{burrito_cfg::BurritoCfg, log_event::{EventType, LogEvent}};


pub struct EventFilter {
    cfg: BurritoCfg,
    event_types: HashSet<EventType>,
}

impl EventFilter {
    pub fn new(cfg: BurritoCfg) -> Self {
        let mut event_types = HashSet::new();
        cfg.rule_list.rules().map(|r| r.get_event_type()).for_each(|t| _ = event_types.insert(t));
        EventFilter {
            cfg: cfg,
            event_types,
        }
    }
    pub fn get_event_types(&self) -> &HashSet<EventType> {
        &self.event_types
    }
    pub fn event_types(&self) -> impl Iterator<Item = &EventType> {
        self.event_types.iter()
    }
    pub fn pass(&self, events: impl Iterator<Item = LogEvent>) {
        events.filter(|e| self.event_types.contains(&e.event_type)).for_each(|event| {
            //TODO: HERE!
        });
    }
}
