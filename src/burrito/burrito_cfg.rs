use serde_derive::{Deserialize, Serialize};

use super::serde_utils;

#[derive(Clone, Deserialize, Serialize)]
pub struct BurritoCfg {
    #[serde(default)]
    pub log_dir: String,
    #[serde(default)]
    pub log_update_interval_ms: u64,
    #[serde(default)]
    pub neut_range_alert_thtd_jumps: u32,
    #[serde(default)]
    pub game_log_alert_cd_ms: u64,
    /*#[serde(default)]
    pub sound_config: AudioAlertConfig,*/
}

impl BurritoCfg {

    pub fn load_from_file() -> Self {
        serde_utils::read_or_create_default_data_struct("", "burrito.cfg")
    }

}

impl Default for BurritoCfg {
    fn default() -> Self {
        Self {
            log_dir: "Documents/Eve/logs/".to_owned(),
            log_update_interval_ms: 500,
            neut_range_alert_thtd_jumps: 5,
            game_log_alert_cd_ms: 5000,
        }
    }
}
