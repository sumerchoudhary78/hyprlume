//! Only libpolkit-agent runs the authentication helper and reports authorization.
use super::wire::{label, Response};
use anyhow::Result;
use polkit_agent_rs::{polkit::UnixUser, Session};
use std::cell::RefCell;
use tokio::sync::{mpsc, oneshot};
use zeroize::Zeroizing;

pub(super) enum Event {
    Prompt(String, bool),
    Notice(String, bool),
    Finished(bool),
}

pub(super) struct Attempt {
    pub responses: mpsc::Sender<Response>,
    pub events: mpsc::Receiver<Event>,
}

impl Attempt {
    pub(super) fn start(uid: i32, cookie: Zeroizing<String>) -> Result<Self> {
        let (responses, mut answers) = mpsc::channel::<Response>(1);
        let (events, results) = mpsc::channel(16);
        std::thread::Builder::new()
            .name("polkit-pam".into())
            .spawn(move || {
                let context = glib::MainContext::new();
                let result = context.with_thread_default(|| {
                    context.block_on(async {
                let session = Session::new(&UnixUser::new(uid), &cookie);
                let (finish, mut finished) = oneshot::channel();
                let finish = RefCell::new(Some(finish));
                session.connect_completed(move |_, success| {
                    if let Some(finish) = finish.take() {
                        let _ = finish.send(success);
                    }
                });
                let sender = events.clone();
                session.connect_request(move |_, text, echo| {
                    let _ = sender.try_send(Event::Prompt(label(text, 192), echo));
                });
                let sender = events.clone();
                session.connect_show_info(move |_, text| {
                    let _ = sender.try_send(Event::Notice(label(text, 280), false));
                });
                let sender = events.clone();
                session.connect_show_error(move |_, text| {
                    let _ = sender.try_send(Event::Notice(label(text, 280), true));
                });
                session.initiate();
                loop {
                    tokio::select! {
                        biased;
                        result = &mut finished => break result.unwrap_or(false),
                        response = answers.recv() => {
                            let Some(response) = response else { session.cancel(); break false; };
                            if let Some(value) = response.value() { session.response(value); }
                        }
                    }
                }
            })
                });
                let _ = events.blocking_send(Event::Finished(result.unwrap_or(false)));
            })?;
        Ok(Self {
            responses,
            events: results,
        })
    }
}
