use serde::{Deserialize, Serialize};

use super::serde_utils;

#[derive(Clone, Deserialize, Serialize)]
pub struct BurritoData {
    #[serde(default)]
    pub faction_npc_alerts: Vec<String>,
    #[serde(default)]
    pub officer_npc_alerts: Vec<String>,
    #[serde(default)]
    pub special_npc_alerts: Vec<String>,
}

impl BurritoData {
    pub fn load_from_file() -> Self {
        serde_utils::read_or_create_default_data_struct("", "burrito.dat")
    }
}

impl Default for BurritoData {
    fn default() -> Self {
        Self {
            faction_npc_alerts: [
                "Dark Blood".to_owned(),
                "Domination".to_owned(),
                "Dread Guristas".to_owned(),
                "Shadow Serpentis".to_owned(),
                "True Sansha".to_owned(),
                "Veles".to_owned(),
            ].to_vec(),
            officer_npc_alerts: [
                // Angel Cartel
                "Gotan Kreiss".to_owned(),
                "Hakim Stormare".to_owned(),
                "Mizuro Cybon".to_owned(),
                "Tobias Kruzhor".to_owned(),
                // Blood Raider Covenant
                "Ahremen Arkah".to_owned(),
                "Draclira Merlonne".to_owned(),
                "Raysere Giant".to_owned(),
                "Tairei Namazoth".to_owned(),
                // Guristas Pirates
                "Estamel Tharchon".to_owned(),
                "Kaikka Peunato".to_owned(),
                "Thon Eney".to_owned(),
                "Vepas Minimala".to_owned(),
                // Sansha's Nation
                "Brokara Ryver".to_owned(),
                "Chelm Soran".to_owned(),
                "Selynne Mardakar".to_owned(),
                "Vizan Ankonin".to_owned(),
                // Serpentis
                "Brynn Jerdola".to_owned(),
                "Cormack Vaaja".to_owned(),
                "Setele Schellan".to_owned(),
                "Tuvan Orth".to_owned(),
            ].to_vec(),
            special_npc_alerts: [
                "Carrier".to_owned(),
                "Dreadnought".to_owned(),
                "Titan".to_owned(),
            ].to_vec(),
        }
    }
}
