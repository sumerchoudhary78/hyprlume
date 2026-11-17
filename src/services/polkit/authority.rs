//! D-Bus admission on an authenticated authority connection.
use super::wire::label;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::{mpsc, oneshot, watch};
use zbus::{message::Header, names::OwnedUniqueName, zvariant::OwnedValue};
use zeroize::Zeroizing;

pub(super) const SERVICE: &str = "org.freedesktop.PolicyKit1";
pub(super) const PATH: &str = "/org/freedesktop/PolicyKit1/Authority";
pub(super) const AGENT_PATH: &str = "/org/hyprlume/PolicyKit1/AuthenticationAgent";
pub(super) const INTERFACE: &str = "org.freedesktop.PolicyKit1.Authority";

type Identity = (String, HashMap<String, OwnedValue>);
type Reply = std::result::Result<(), AgentError>;

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.PolicyKit1.Error")]
pub(super) enum AgentError {
    Failed(String),
    Cancelled(String),
    NotAuthorized(String),
    #[zbus(error)]
    Bus(zbus::Error),
}

pub(super) struct Challenge {
    pub action: String,
    pub message: String,
    pub cookie: Zeroizing<String>,
    pub users: Vec<i32>,
    pub cancelled: watch::Receiver<bool>,
    pub reply: oneshot::Sender<Reply>,
}

pub(super) struct Agent {
    owner: OwnedUniqueName,
    requests: mpsc::Sender<Challenge>,
    active: Mutex<Option<Active>>,
}

struct Active {
    cookie: Zeroizing<String>,
    cancel: watch::Sender<bool>,
}

impl Agent {
    pub(super) fn new(owner: OwnedUniqueName, requests: mpsc::Sender<Challenge>) -> Self {
        Self {
            owner,
            requests,
            active: Mutex::new(None),
        }
    }

    fn check_sender(&self, header: &Header<'_>) -> Reply {
        if header.sender().map(|name| name.as_str()) == Some(self.owner.as_str()) {
            return Ok(());
        }
        Err(AgentError::NotAuthorized(
            "Polkit authority required".into(),
        ))
    }

    fn admit(
        &self,
        cookie: &str,
        cancel: watch::Sender<bool>,
    ) -> std::result::Result<Admission<'_>, AgentError> {
        let mut active = self.active.lock().unwrap();
        if active.is_some() {
            return Err(AgentError::Failed(
                "Another authentication request is open".into(),
            ));
        }
        *active = Some(Active {
            cookie: Zeroizing::new(cookie.to_owned()),
            cancel,
        });
        Ok(Admission(self))
    }
}

struct Admission<'a>(&'a Agent);
impl Drop for Admission<'_> {
    fn drop(&mut self) {
        self.0.active.lock().unwrap().take();
    }
}

#[zbus::interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
impl Agent {
    #[allow(clippy::too_many_arguments, reason = "Polkit's D-Bus signature")]
    async fn begin_authentication(
        &self,
        action_id: String,
        message: String,
        _icon_name: String,
        _details: HashMap<String, String>,
        cookie: String,
        identities: Vec<Identity>,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), AgentError> {
        let cookie = Zeroizing::new(cookie);
        self.check_sender(&header)?;
        let mut users = Vec::new();
        for (kind, properties) in identities.into_iter().take(32) {
            if kind == "unix-user" {
                if let Some(uid) = properties
                    .get("uid")
                    .and_then(|value| u32::try_from(value).ok())
                    .and_then(|uid| i32::try_from(uid).ok())
                {
                    if !users.contains(&uid) {
                        users.push(uid);
                    }
                }
            }
        }
        if users.is_empty() || cookie.is_empty() || cookie.len() > 1024 {
            return Err(AgentError::Failed("Unsupported Polkit challenge".into()));
        }
        let (cancel, cancelled) = watch::channel(false);
        let _admission = self.admit(&cookie, cancel)?;
        let (reply, result) = oneshot::channel();
        self.requests
            .try_send(Challenge {
                action: label(&action_id, 192),
                message: label(&message, 360),
                cookie,
                users,
                cancelled,
                reply,
            })
            .map_err(|_| AgentError::Failed("Authentication dialog unavailable".into()))?;
        result
            .await
            .map_err(|_| AgentError::Failed("Authentication dialog disconnected".into()))?
    }

    async fn cancel_authentication(
        &self,
        cookie: String,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), AgentError> {
        let cookie = Zeroizing::new(cookie);
        self.check_sender(&header)?;
        if let Some(active) = self
            .active
            .lock()
            .unwrap()
            .as_ref()
            .filter(|active| *active.cookie == *cookie)
        {
            active.cancel.send_replace(true);
        }
        Ok(())
    }
}
