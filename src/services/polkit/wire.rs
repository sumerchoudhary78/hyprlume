use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, Read, Write},
    os::unix::fs::FileTypeExt,
};
use tokio::sync::mpsc;
use zeroize::{Zeroize, Zeroizing};

const FRAME_BYTES: u64 = 8192;

#[derive(Deserialize)]
#[serde(transparent)]
pub(super) struct Response(String);

impl Drop for Response {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Response {
    pub(super) fn value(&self) -> Option<&str> {
        (self.0.len() <= 1024 && !self.0.contains(['\r', '\n', '\0'])).then_some(&self.0)
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Input {
    Available {
        value: bool,
    },
    Start {
        id: u32,
        account: usize,
    },
    Respond {
        id: u32,
        prompt: u32,
        response: Response,
    },
    Cancel {
        id: u32,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Output<'a> {
    Registered,
    Open {
        id: u32,
        action: &'a str,
        message: &'a str,
        accounts: &'a [String],
    },
    Prompt {
        id: u32,
        prompt: u32,
        label: &'a str,
        echo: bool,
    },
    Notice {
        id: u32,
        message: &'a str,
        error: bool,
    },
    Retry {
        id: u32,
    },
    Close {
        id: u32,
    },
}

pub(super) fn send(output: Output<'_>) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &output)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub(super) fn receive() -> Result<mpsc::Receiver<Input>> {
    for fd in [0, 1] {
        ensure!(
            std::fs::metadata(format!("/proc/self/fd/{fd}"))?
                .file_type()
                .is_fifo(),
            "The Polkit worker needs private UI pipes"
        );
    }
    let (sender, receiver) = mpsc::channel(8);
    std::thread::Builder::new()
        .name("polkit-ui-input".into())
        .spawn(move || {
            let mut stdin = std::io::stdin().lock();
            let mut line = Zeroizing::new(Vec::with_capacity(FRAME_BYTES as usize));
            loop {
                line.clear();
                match stdin
                    .by_ref()
                    .take(FRAME_BYTES + 1)
                    .read_until(b'\n', &mut line)
                {
                    Ok(n) if n > 0 && n as u64 <= FRAME_BYTES && line.ends_with(b"\n") => (),
                    _ => break,
                }
                let input = serde_json::from_slice(&line);
                line.zeroize();
                let Ok(input) = input else {
                    break;
                };
                if sender.blocking_send(input).is_err() {
                    break;
                }
            }
        })?;
    Ok(receiver)
}

pub(super) fn label(text: &str, limit: usize) -> String {
    text.chars()
        .filter(|ch| {
            !ch.is_control() && !matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(limit)
        .collect()
}
