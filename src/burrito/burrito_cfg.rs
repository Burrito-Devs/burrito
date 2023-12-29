use std::collections::{BTreeSet, HashSet};

use serde_derive::{Deserialize, Serialize};

use super::{serde_utils, utils, log_watcher::IntelChannel, log_event::EventType};

#[derive(Clone, Deserialize, Serialize)]
pub struct BurritoCfg {
    #[serde(default)]
    pub log_dir: String,
    #[serde(default)]
    pub log_update_interval_ms: u64,
    #[serde(default)]
    pub game_log_alert_cd_ms: u64,
    #[serde(default)]
    pub hide_chat_messages: bool,
    #[serde(default)]
    pub hide_out_of_range_events: bool,
    #[serde(default)]
    pub recent_post_cache_ttl_ms: i64,
    #[serde(default)]
    pub sound_config: AudioAlertConfig,
    #[serde(default)]
    pub text_channel_config: TextChannelConfig,
}

impl BurritoCfg {

    pub fn load_from_file() -> Self {
        serde_utils::read_or_create_default_data_struct("", "burrito.cfg")
    }

}

impl Default for BurritoCfg {
    fn default() -> Self {
        Self {
            log_dir: format!("{}/Documents/Eve/logs/", utils::get_home_dir()).to_owned(),
            log_update_interval_ms: 500,
            game_log_alert_cd_ms: 5000,
            hide_chat_messages: false,
            hide_out_of_range_events: false,
            recent_post_cache_ttl_ms: 30000,
            sound_config: Default::default(),
            text_channel_config: Default::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AudioAlertConfig {
    #[serde(default)]
    pub audio_alerts: BTreeSet<AudioAlert>,
}

impl Default for AudioAlertConfig {
    fn default() -> Self {
        let mut def = Self {
            audio_alerts: BTreeSet::new(),
        };
        let burrito_dir = utils::get_burrito_dir();
        let mut def_neutral_file = burrito_dir.clone();
        def_neutral_file.push_str("sounds/neut_in_range.mp3");
        /*def.audio_alerts.insert(AudioAlert {// TODO: HERE!
            trigger: EventType::RangeOfSystem(5),
            sound_file: def_neutral_file,
        });*/
        let mut def_faction_file = burrito_dir.clone();
        def_faction_file.push_str("sounds/faction_spawn.mp3");
        def.audio_alerts.insert(AudioAlert {
            trigger: EventType::FactionSpawn,
            sound_file: def_faction_file,
        });
        let mut def_special_spawn = burrito_dir.clone();
        def_special_spawn.push_str("sounds/special_spawn.mp3");
        def.audio_alerts.insert(AudioAlert {
            trigger: EventType::DreadSpawn,
            sound_file: def_special_spawn.clone(),
        });
        def.audio_alerts.insert(AudioAlert {
            trigger: EventType::TitanSpawn,
            sound_file: def_special_spawn.clone(),
        });
        def.audio_alerts.insert(AudioAlert {
            trigger: EventType::OfficerSpawn,
            sound_file: def_special_spawn,
        });
        def
    }
}

#[derive(Clone, Debug, Eq, Hash, Deserialize, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AudioAlert {
    pub trigger: EventType,
    pub sound_file: String,
}

#[derive(Clone, Debug, Eq, Hash, Deserialize, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TextAlert {
    pub trigger: EventType,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TextChannelConfig {
    #[serde(default)]
    pub text_channels: HashSet<IntelChannel>,
}

impl Default for TextChannelConfig {
    fn default() -> Self {
        let mut channels = HashSet::new();
        channels.insert(IntelChannel::Delve);
        channels.insert(IntelChannel::Querious);
        TextChannelConfig { text_channels: channels }
    }
}
