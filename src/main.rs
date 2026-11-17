use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hyprlume::{
    config::Config,
    protocol::{Operation, VoiceMode},
    server,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[arg(long, global = true)]
    socket: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the event-driven daemon. Microphone capture starts OFF.
    Daemon {
        #[arg(long)]
        demo: bool,
    },
    /// Start the Quickshell UI; --preview runs in an ordinary window.
    Shell {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        preview: bool,
    },
    /// Run the desktop authorization agent for this Wayland login.
    Polkit {
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Inspect this session's desktop portals or print the Hyprlume profile.
    Portals {
        #[arg(long)]
        profile: bool,
    },
    #[command(hide = true)]
    PortalSettings,
    #[command(hide = true)]
    PolkitWorker,
    /// Preview a command; --execute runs a direct command immediately.
    Send {
        text: String,
        #[arg(long)]
        execute: bool,
    },
    Status,
    /// Open the wireless panel, or print its current state without changing connections.
    Connectivity {
        #[arg(long)]
        status: bool,
    },
    /// Open the notification centre, or read its state without changing notifications.
    Notifications {
        #[arg(long)]
        status: bool,
    },
    /// Open the sound and media panel without changing any audio settings.
    Audio,
    /// Open application tray icons and their menus.
    Tray,
    /// Search installed applications; --windows opens the task switcher.
    Launcher {
        #[arg(long)]
        windows: bool,
    },
    /// Open power/session controls, or read the current state without changing power.
    Power {
        #[arg(long)]
        status: bool,
    },
    /// Open clipboard history, or deliver an offer from wl-paste.
    Clipboard {
        #[command(subcommand)]
        action: Option<ClipboardCommand>,
    },
    /// Open screen capture; --stop finalizes the owned recording.
    Capture {
        #[arg(long)]
        stop: bool,
    },
    /// Open persisted appearance, voice/model and service preferences.
    Settings,
    /// Lock the session; --wait returns only after compositor acknowledgment.
    Lock {
        #[arg(long)]
        wait: bool,
    },
    #[command(hide = true)]
    LockWorker,
    #[command(hide = true)]
    IdleEvent {
        #[arg(value_enum)]
        event: hyprlume::services::idle::Event,
    },
    /// Read idle policy status or temporarily pause automatic timers.
    Idle {
        #[arg(long, value_enum)]
        presentation: Option<Toggle>,
    },
    /// Inspect or switch the active keyboard layout.
    Input {
        #[arg(long)]
        next_layout: bool,
    },
    /// Execute a system volume, brightness or media key.
    SystemKey {
        #[arg(value_enum)]
        key: hyprlume::desktop::input::SystemKey,
    },
    Toggle,
    Cancel,
    /// Confirm a specific preview token (expires after 30 seconds).
    Confirm {
        token: u64,
    },
    /// Enable wake-word commands or dictation, or turn capture off.
    Voice {
        #[arg(value_enum)]
        mode: VoiceMode,
    },
    /// Check prerequisites without recording audio or changing the desktop.
    Doctor,
    /// Print a complete starting configuration to stdout.
    Config,
    #[cfg(feature = "voice")]
    /// Transcribe a prerecorded WAV without using the microphone.
    Transcribe {
        file: PathBuf,
    },
    #[cfg(feature = "voice")]
    /// Synthesize English speech to a WAV; does not play it.
    Speak {
        text: String,
        #[arg(long)]
        output: PathBuf,
    },
    #[cfg(feature = "voice")]
    /// Test keyword detection on a prerecorded WAV.
    DetectWake {
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum ClipboardCommand {
    #[command(hide = true)]
    Ingest {
        #[arg(long)]
        generation: u64,
    },
}
#[derive(Clone, Copy, clap::ValueEnum)]
enum Toggle {
    On,
    Off,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(hyprlume::config::config_path);
    let needs_config = match &cli.command {
        Commands::Daemon { .. } | Commands::Config | Commands::Doctor => true,
        #[cfg(feature = "voice")]
        Commands::Transcribe { .. } | Commands::Speak { .. } | Commands::DetectWake { .. } => true,
        _ => false,
    };
    // Socket clients, especially per-offer clipboard delivery, do not need
    // inference configuration or unrelated config-file I/O.
    let config = if needs_config {
        Config::load(cli.config)?
    } else {
        Config::default()
    };
    match cli.command {
        Commands::Portals { profile: true } => {
            print!("{}", hyprlume::services::portals::PROFILE);
        }
        Commands::Portals { profile: false } => {
            let status = hyprlume::services::portals::inspect(false).await;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Commands::PortalSettings => {
            return hyprlume::services::portals::serve_appearance(config_path, cli.socket).await;
        }
        Commands::IdleEvent { event } => return hyprlume::services::idle::callback(event).await,
        Commands::Idle { presentation } => {
            let operation = match presentation {
                Some(value) => Operation::Idle {
                    request: hyprlume::services::idle::Request::Presentation {
                        enabled: matches!(value, Toggle::On),
                    },
                },
                None => Operation::IdleStatus,
            };
            let socket = cli.socket.unwrap_or(server::socket_path()?);
            println!(
                "{}",
                serde_json::to_string_pretty(&server::call(&socket, operation).await?)?
            );
        }
        Commands::Config => {
            println!("{}", toml::to_string_pretty(&config)?);
        }
        Commands::Shell { path, preview } => {
            let path = path.unwrap_or_else(|| PathBuf::from("/usr/share/hyprlume/shell/shell.qml"));
            let path = if path.is_dir() {
                path.join("shell.qml")
            } else {
                path
            };
            anyhow::ensure!(
                path.is_file(),
                "shell not found at {}; use --path ./shell when developing",
                path.display()
            );
            let mut command = std::process::Command::new("qs");
            command.arg("--path").arg(path);
            if preview {
                command.env("HYPRLUME_PREVIEW", "1");
            }
            if let Some(socket) = cli.socket {
                command.env("HYPRLUME_SOCKET", socket);
            }
            use std::os::unix::process::CommandExt;
            return Err(command.exec()).context("start Quickshell (install quickshell)");
        }
        Commands::Polkit { path } => return hyprlume::services::polkit::launch(path, cli.socket),
        Commands::PolkitWorker => return hyprlume::services::polkit::worker().await,
        Commands::LockWorker => return hyprlume::services::locking::run_worker().await,
        Commands::Doctor => {
            println!(
                "Hyprlume {} · microphone remains off",
                env!("CARGO_PKG_VERSION")
            );
            println!(
                "Hyprland: {}",
                if hyprpilot_core::Connection::new().is_ok() {
                    "own session found"
                } else {
                    "not found in this session"
                }
            );
            for tool in [
                "qs",
                "pw-record",
                "pw-play",
                "wpctl",
                "espeak-ng",
                "gtk-launch",
                "wl-paste",
                "wl-copy",
                "grim",
                "slurp",
                "wf-recorder",
                "swaylock",
            ] {
                let found = std::env::var_os("PATH")
                    .is_some_and(|p| std::env::split_paths(&p).any(|dir| dir.join(tool).is_file()));
                println!("{tool}: {}", if found { "found" } else { "missing" });
            }
            #[cfg(feature = "voice")]
            for mode in [VoiceMode::Wake, VoiceMode::Dictate] {
                println!(
                    "voice {mode:?}: {}",
                    hyprlume::voice::check_models(&config.voice, mode)
                        .map(|_| "ready".to_owned())
                        .unwrap_or_else(|e| e.to_string())
                );
            }
        }
        #[cfg(feature = "voice")]
        Commands::Transcribe { file } => {
            println!("{}", hyprlume::voice::transcribe(&config.voice, &file)?)
        }
        #[cfg(feature = "voice")]
        Commands::Speak { text, output } => hyprlume::voice::speak(&config.voice, &text, &output)?,
        #[cfg(feature = "voice")]
        Commands::DetectWake { file } => {
            println!("{}", hyprlume::voice::detect_wake(&config.voice, &file)?)
        }
        command => {
            let wait_for_lock = matches!(&command, Commands::Lock { wait: true });
            let path = cli.socket.map(Ok).unwrap_or_else(server::socket_path)?;
            let operation = match command {
                Commands::Daemon { demo } => {
                    return server::serve_with_config(path, config, demo, config_path).await
                }
                Commands::Send { text, execute } => Operation::Command {
                    text,
                    dry_run: !execute,
                },
                Commands::Status => Operation::Status,
                Commands::Connectivity { status } => {
                    if status {
                        Operation::ConnectivityStatus
                    } else {
                        Operation::Connectivity {
                            request: hyprlume::services::connectivity::Request::Toggle,
                        }
                    }
                }
                Commands::Notifications { status } => {
                    if status {
                        Operation::NotificationsStatus
                    } else {
                        Operation::Notifications {
                            request: hyprlume::services::notifications::Request::Toggle,
                        }
                    }
                }
                Commands::Clipboard {
                    action: Some(ClipboardCommand::Ingest { generation }),
                } => {
                    let Some(text) = hyprlume::services::clipboard::read_offer()? else {
                        return Ok(());
                    };
                    let operation = Operation::ClipboardIngest { generation, text };
                    anyhow::ensure!(
                        serde_json::to_vec(&hyprlume::protocol::Request {
                            id: 1,
                            operation: operation.clone()
                        })?
                        .len()
                            < server::MAX_FRAME,
                        "Clipboard offer exceeds the wire limit"
                    );
                    operation
                }
                Commands::Clipboard { action: None } => Operation::Panel {
                    panel: hyprlume::protocol::DesktopPanel::Clipboard,
                    open: true,
                },
                Commands::Capture { stop: true } => Operation::Capture {
                    request: hyprlume::services::capture::Request::Stop,
                },
                Commands::Capture { stop: false } => Operation::Panel {
                    panel: hyprlume::protocol::DesktopPanel::Capture,
                    open: true,
                },
                Commands::Lock { .. } => Operation::Lock,
                Commands::Settings => Operation::Panel {
                    panel: hyprlume::protocol::DesktopPanel::Settings,
                    open: true,
                },
                Commands::Input { next_layout } => Operation::Input {
                    request: if next_layout {
                        hyprlume::desktop::input::Request::NextLayout
                    } else {
                        hyprlume::desktop::input::Request::Status
                    },
                },
                Commands::SystemKey { key } => Operation::SystemKey { key },
                Commands::Toggle => Operation::Toggle,
                Commands::Audio => Operation::Panel {
                    panel: hyprlume::protocol::DesktopPanel::Audio,
                    open: true,
                },
                Commands::Tray => Operation::Panel {
                    panel: hyprlume::protocol::DesktopPanel::Tray,
                    open: true,
                },
                Commands::Launcher { windows } => Operation::Panel {
                    panel: if windows {
                        hyprlume::protocol::DesktopPanel::Switcher
                    } else {
                        hyprlume::protocol::DesktopPanel::Launcher
                    },
                    open: true,
                },
                Commands::Power { status } => {
                    if status {
                        Operation::PowerStatus
                    } else {
                        Operation::Panel {
                            panel: hyprlume::protocol::DesktopPanel::Power,
                            open: true,
                        }
                    }
                }
                Commands::Cancel => Operation::Cancel,
                Commands::Confirm { token } => Operation::Confirm { token },
                Commands::Voice { mode } => Operation::Voice { mode },
                _ => unreachable!(),
            };
            let response = server::call(&path, operation).await?;
            if wait_for_lock {
                server::wait_for_lock(&path).await?;
            }
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }
    Ok(())
}
