use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedProfile {
    pub code: String,
    pub profile: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}
