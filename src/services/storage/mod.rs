//! On-demand removable storage controls provided by UDisks2.
#[cfg(target_os = "linux")]
pub(crate) mod native;

use crate::services::connectivity::Secret;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Refresh,
    Mount { id: String },
    Unmount { id: String },
    Open { id: String },
    Unlock { id: String, password: Secret },
    Lock { id: String },
    Eject { id: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct State {
    pub available: bool,
    pub busy: bool,
    pub message: String,
    pub error: String,
    pub drives: Vec<Drive>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Drive {
    id: String,
    label: String,
    can_eject: bool,
    volumes: Vec<Volume>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Volume {
    id: String,
    label: String,
    filesystem: bool,
    mounted: bool,
    encrypted: bool,
    locked: bool,
    protected: bool,
}
