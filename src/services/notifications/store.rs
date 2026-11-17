//! Bounded, memory-only notification lifecycle. No timers, I/O or UI code.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]
use super::types::{Action, Incoming, Notice, Request, Signal, State};
use anyhow::{bail, Context, Result};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(super) const MAX_ITEMS: usize = 24;
const MAX_POPUPS: usize = 3;

struct Record {
    notice: Notice,
    owner: String,
    expires: Option<Instant>,
    transient: bool,
    resident: bool,
}

#[derive(Default)]
pub(super) struct Store {
    records: Vec<Record>,
    next_id: u32,
    revision: u64,
    pub state: State,
}

fn text(input: &str, max_bytes: usize) -> String {
    let mut output = String::new();
    for ch in input.chars().filter(|ch| !ch.is_control() || *ch == '\n') {
        if output.len() + ch.len_utf8() > max_bytes {
            break;
        }
        output.push(ch);
    }
    output
}

impl Store {
    pub fn offline(&mut self, message: &str) {
        self.state.available = false;
        self.state.error = message.into();
        for record in &mut self.records {
            record.notice.active = false;
            record.notice.popup = false;
            record.expires = None;
        }
    }

    pub fn snapshot(&self) -> State {
        State {
            items: self
                .records
                .iter()
                .map(|record| record.notice.clone())
                .collect(),
            unread: self
                .records
                .iter()
                .filter(|record| !record.notice.read)
                .count(),
            ..self.state.clone()
        }
    }

    pub fn receive(
        &mut self,
        owner: &str,
        input: Incoming,
        now: Instant,
    ) -> Result<(u32, Vec<Signal>)> {
        anyhow::ensure!(
            input.summary.len() <= 8192
                && input.body.len() <= 32768
                && input.app_name.len() <= 1024,
            "Notification text exceeds input limits"
        );
        anyhow::ensure!(
            input.actions.len() <= 32 && input.actions.len().is_multiple_of(2),
            "Notification actions must be key/label pairs"
        );
        anyhow::ensure!(input.timeout_ms >= -1, "Invalid notification timeout");
        let actions = input
            .actions
            .as_chunks::<2>()
            .0
            .iter()
            .filter(|pair| {
                !pair[0].is_empty() && pair[0].len() <= 64 && !pair[0].chars().any(char::is_control)
            })
            .take(3)
            .map(|pair| Action {
                key: pair[0].clone(),
                label: text(&pair[1], 48),
            })
            .collect();
        let replace = self.records.iter().position(|record| {
            record.notice.id == input.replaces_id && record.notice.active && record.owner == owner
        });
        let id = if let Some(index) = replace {
            self.records.remove(index).notice.id
        } else {
            loop {
                self.next_id = self.next_id.wrapping_add(1).max(1);
                if !self
                    .records
                    .iter()
                    .any(|record| record.notice.id == self.next_id)
                {
                    break self.next_id;
                }
            }
        };
        self.revision = self.revision.wrapping_add(1);
        let timeout_ms = if input.urgency >= 2 || input.resident || input.timeout_ms == 0 {
            None
        } else if input.timeout_ms == -1 {
            Some(7000)
        } else {
            Some(input.timeout_ms as u64)
        };
        let popup = !self.state.dnd && !self.state.panel_open;
        self.records.insert(
            0,
            Record {
                notice: Notice {
                    id,
                    revision: self.revision,
                    app_name: text(&input.app_name, 64),
                    summary: text(&input.summary, 128),
                    body: text(&input.body, 512),
                    actions,
                    urgency: input.urgency.min(2),
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    active: true,
                    popup,
                    read: self.state.panel_open,
                },
                owner: owner.into(),
                expires: timeout_ms.map(|ms| now + Duration::from_millis(ms)),
                transient: input.transient,
                resident: input.resident,
            },
        );
        // Older popups remain in the centre rather than producing an unbounded stack.
        let mut visible = 0;
        for record in &mut self.records {
            if record.notice.popup {
                visible += 1;
                if visible > MAX_POPUPS {
                    record.notice.popup = false;
                }
            }
        }
        let mut signals = Vec::new();
        if self.records.len() > MAX_ITEMS {
            let removed = self.records.pop().unwrap();
            if removed.notice.active {
                signals.push(Signal::Closed {
                    owner: removed.owner,
                    id: removed.notice.id,
                    reason: 4,
                });
            }
        }
        Ok((id, signals))
    }

    fn finish(&mut self, index: usize, reason: u32, remove: bool) -> Vec<Signal> {
        let record = &mut self.records[index];
        let signals = if record.notice.active {
            vec![Signal::Closed {
                owner: record.owner.clone(),
                id: record.notice.id,
                reason,
            }]
        } else {
            Vec::new()
        };
        record.notice.active = false;
        record.notice.popup = false;
        record.expires = None;
        if reason == 2 {
            record.notice.read = true;
        }
        if remove || record.transient {
            self.records.remove(index);
        }
        signals
    }

    pub fn close_from_client(&mut self, owner: &str, id: u32) -> Result<Vec<Signal>> {
        let index = self
            .records
            .iter()
            .position(|record| {
                record.notice.active && record.notice.id == id && record.owner == owner
            })
            .context("No active notification belongs to this sender")?;
        Ok(self.finish(index, 3, true))
    }

    pub fn request(&mut self, request: Request) -> Result<Vec<Signal>> {
        match request {
            Request::Toggle => {
                self.state.panel_open = !self.state.panel_open;
                if self.state.panel_open {
                    for record in &mut self.records {
                        record.notice.read = true;
                        record.notice.popup = false;
                    }
                }
            }
            Request::Close => self.state.panel_open = false,
            Request::Dnd { enabled } => {
                self.state.dnd = enabled;
                if enabled {
                    for record in &mut self.records {
                        record.notice.popup = false;
                    }
                }
            }
            Request::Clear => {
                let mut signals = Vec::new();
                while !self.records.is_empty() {
                    signals.extend(self.finish(0, 2, true));
                }
                return Ok(signals);
            }
            request => {
                let (id, revision) = match &request {
                    Request::Dismiss { id, revision }
                    | Request::Remove { id, revision }
                    | Request::Invoke { id, revision, .. } => (*id, *revision),
                    _ => unreachable!(),
                };
                let index = self
                    .records
                    .iter()
                    .position(|record| record.notice.id == id && record.notice.revision == revision)
                    .context("Notification was replaced or removed")?;
                match request {
                    Request::Dismiss { .. } => return Ok(self.finish(index, 2, false)),
                    Request::Remove { .. } => return Ok(self.finish(index, 2, true)),
                    Request::Invoke { key, .. } => {
                        let record = &mut self.records[index];
                        anyhow::ensure!(
                            record.notice.active
                                && record.notice.actions.iter().any(|action| action.key == key),
                            "Notification action is no longer available"
                        );
                        record.notice.read = true;
                        let mut signals = vec![Signal::Action {
                            owner: record.owner.clone(),
                            id,
                            key,
                        }];
                        if !record.resident {
                            signals.extend(self.finish(index, 2, false));
                        }
                        return Ok(signals);
                    }
                    _ => bail!("Invalid notification request"),
                }
            }
        }
        Ok(Vec::new())
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.records
            .iter()
            .filter_map(|record| record.expires)
            .min()
    }

    pub fn expire(&mut self, now: Instant) -> Vec<Signal> {
        let mut signals = Vec::new();
        for index in (0..self.records.len()).rev() {
            if self.records[index]
                .expires
                .is_some_and(|deadline| deadline <= now)
            {
                signals.extend(self.finish(index, 1, false));
            }
        }
        signals
    }
}
