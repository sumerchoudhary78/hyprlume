use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Toggle,
    Close,
    Dnd { enabled: bool },
    Dismiss { id: u32, revision: u64 },
    Remove { id: u32, revision: u64 },
    Invoke { id: u32, revision: u64, key: String },
    Clear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notice {
    pub id: u32,
    pub revision: u64,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<Action>,
    pub urgency: u8,
    pub created_at: u64,
    pub active: bool,
    pub popup: bool,
    pub read: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub available: bool,
    pub error: String,
    pub panel_open: bool,
    pub dnd: bool,
    pub unread: usize,
    pub items: Vec<Notice>,
}

/// D-Bus input stays outside the AI protocol and carries no executable command.
#[derive(Default)]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(super) struct Incoming {
    pub app_name: String,
    pub replaces_id: u32,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub urgency: u8,
    pub timeout_ms: i32,
    pub transient: bool,
    pub resident: bool,
}

#[derive(Debug, PartialEq)]
pub(super) enum Signal {
    Closed { owner: String, id: u32, reason: u32 },
    Action { owner: String, id: u32, key: String },
}
