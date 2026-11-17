# Architecture

## Source map

```text
src/
├── main.rs                   CLI arguments and process entry
├── lib.rs                    public module surface
├── config.rs                 configuration validation and XDG paths
├── config/preferences.rs     bounded appearance, shortcut and startup settings
├── protocol.rs               shared wire operations and state snapshots
├── app/
│   ├── mod.rs                state ownership and operation routing
│   ├── commands.rs           admission, planning, review and execution lifecycle
│   ├── events.rs             compositor observations
│   ├── locking.rs            epoch revocation and coordinated service pausing
│   └── voice.rs              voice lifecycle, event handling and feedback
├── server/
│   ├── mod.rs                bounded sessions, subscriptions and daemon lifetime
│   ├── socket.rs             socket permissions, file lock and cleanup
│   └── client.rs             one-shot CLI requests
├── planner/
│   ├── mod.rs                typed actions, plan validation and planning policy
│   ├── direct.rs             deterministic English command parsing
│   └── llm.rs                bounded loopback HTTP adapter and response schema
├── desktop/
│   ├── mod.rs                validated action routing and bounded process runner
│   ├── hyprland.rs           workspace/window actions and Lua compatibility
│   ├── applications.rs       independent systemd application launches
│   └── audio.rs              checked PipeWire volume/mute actions
├── services/
│   ├── connectivity/         wireless state, validation, admission and pairing
│   │   └── native/           NetworkManager/BlueZ adapters and event streams
│   ├── notifications/        bounded lifecycle, D-Bus server, expiry and UI topic
│   ├── power/                native observations, bounded mutations and session review
│   │   └── native/           UPower, backlights, profiles and logind adapters
│   ├── clipboard/            opt-in watcher and bounded volatile text store
│   ├── capture/              owned jobs, validated targets and private storage
│   ├── settings/             versioned drafts, validation and atomic persistence
│   └── locking/              independent notify unit, PAM client and recovery marker
└── voice/
    ├── mod.rs                public speech API and events
    ├── models.rs             model discovery and recognizer configuration
    ├── recognition.rs        PCM conversion, capture and streaming recognition
    └── synthesis.rs          phonemization and Piper synthesis
```

`server` depends on the application; application workflows do not depend on
socket sessions. Both consume the shared protocol/configuration. The application
coordinates concrete planner, desktop and voice adapters. Submodules stay
private unless they form a caller-facing API; `server::App`, `server::call`,
`planner::{Action, Plan, direct, plan}` and `voice::*` retain their existing paths.
The canonical application type is `app::App`.

To add an action, extend its typed representation/validation in `planner`, its
direct phrases and model schema, then its concrete adapter in `desktop`.
Exercise preview/confirmation and observed postconditions in integration tests.
A new speech engine belongs behind the `voice` API; command/review policy stays
in `app`. Native inference belongs on the blocking worker. Additional UI panels
use the existing service interfaces and shared view state; see
[the shell source map](../shell/README.md).

## Processes and ownership

Quickshell owns rendering, input widgets, native PipeWire/MPRIS adapters and layer-shell surfaces. The Rust
daemon owns configuration, the bounded action planner/executor, model work,
and observed desktop state. A CLI sends the same protocol as the UI.

The components communicate over a private Unix socket at
`$XDG_RUNTIME_DIR/hyprlume/control.sock`. Its directory is mode 0700 and the
socket is mode 0600. A file lock prevents a second daemon replacing a live
instance. Only a socket, never a regular file or symlink, can be replaced on
restart. At most 16 clients connect, with bounded frames and write deadlines.

The UI uses one socket with agent, connectivity, notification and power subscriptions. Independent
Tokio watch channels coalesce state into the latest
snapshot rather than retaining an unbounded event queue. Reconnect timers run
only while disconnected. The daemon reads Hyprland's event socket through
HyprPilot, without a polling loop for desktop status.

## Shell surfaces

Idle policy is an opt-in Rust controller with a bounded callback queue and a
single owned swayidle event source. Generation changes cancel queued effects;
brightness and DPMS restoration use identity/value leases. Sleep hooks bypass
the serial idle queue to avoid deadlocking logind's PrepareForSleep signal.
Power's manual and automatic suspend paths share a compositor-confirmed lock
guard. See [idle/sleep](idle-sleep.md) for the limits of external sleep hooks.

Quickshell renders ten layer-shell surface roles and one ordinary window for
preview:

- `hyprlume-wallpaper`, one per monitor, Background layer with no input region.
- `hyprlume-bar`, one per monitor, `WlrLayer.Top`, reserves its scaled visible height at the top
  so Hyprland tiles windows below it.
- `hyprlume-home`, one on the focused monitor, `WlrLayer.Bottom`. It maps only
  while the focused workspace has no windows and unmaps otherwise, so an
  ordinary windowed session pays no rendering cost for it.
- `hyprlume-command`, `WlrLayer.Overlay` with exclusive keyboard focus, loaded
  only while `snapshot.panel_open` is true.
- `hyprlume-connectivity`, a separate Overlay loaded only while Connections is
  open. Command and wireless overlays exclude each other.
- `PreviewWindow`, a `FloatingWindow` used by `hyprlume shell --preview`.
- `hyprlume-notification-centre`, an Overlay with exclusive keyboard focus while
  its centre is open; it excludes command and connectivity overlays.
- `hyprlume-notification-popups`, a small Overlay on the focused screen with no
  keyboard focus or exclusive zone, loaded only while popups are present.
- `hyprlume-audio`, an exclusive-keyboard Overlay for native devices and media,
  coordinated with the other panels through Rust's typed `desktop_panel` state.
- `hyprlume-power`, another exclusive-keyboard Overlay with independent power
  state and a separate session-action review, coordinated by `desktop_panel`.
- `hyprlume-tray`, an exclusive-keyboard Overlay for native StatusNotifier icons
  and themed DBusMenu trees; closed menus release native references.

Surfaces contain no protocol logic. `services/Backend.qml` owns the socket and
the reconnecting subscription; `services/Workspaces.qml` wraps Hyprland state;
`services/NativeAudio.qml` uses native event streams without polling processes;
`services/Metrics.qml` owns the only child process the shell starts; `Clock` and
`ShellState` are local. Modules receive these as required properties, which keeps
every panel usable in isolation and previewable without a daemon.

The dashboard's glass samples its own cached wallpaper within QML. Compositor
blur is optional for the bar/command surfaces; application contents are never
captured by the glass renderer. One Qt Quick 3D viewport holds the globe and city.
The reduced-effects profile skips that viewport and local blur. See
[homescreen](homescreen.md) for composition, responsiveness and data ownership.

Wireless state and commands have a dedicated controller under `services`,
outside the planner. NetworkManager and BlueZ supply event-driven hardware
observations. A semaphore rejects overlapping wireless operations, discovery is
timed, and pairing uses an expiring response broker. Credentials stay out of
published snapshots and AI state. Details and extension points are in
[connectivity](connectivity.md).

## HyprPilot integration

`hyprpilot-core` is an actual Cargo dependency pinned to commit
`de1f0f4ae25aabe3fcf4964150eb33c1386cd9d4` from
[HyprPilot](https://github.com/sumerchoudhary78/HyprPilot).
Queries, selectors, workspace dispatchers, focus, window moves, and compositor
events use that implementation. Hyprlume does not copy a second Hyprland IPC
client into this repository. A small adapter reads `j/status.configProvider`
through that transport and uses bounded Lua dispatchers for modern Lua sessions;
classic sessions retain HyprPilot's typed legacy operations. Window selectors
are validated hexadecimal addresses before becoming Lua literals.

HyprPilot's daemon, persistent undo, snapshots, accessibility, and MCP server
remain useful adjacent integrations. This release does not start or expose
those broader capabilities. They are not required to run Hyprlume, and the
installed `hyprpilot-mcp` package is not a runtime dependency of this shell.

## Planning and execution

1. Validate plain text and attempt direct command parsing.
2. For an unmatched request, use the explicitly configured local model server.
3. Deserialize and validate a plan of at most four known actions. Launch
   names must resolve to trusted configured argv arrays.
4. Preview, with a single-use confirmation token. Model plans always preview.
5. Apply actions serially. Stop the batch on the first error and retain any
   receipts for earlier completed actions.
6. Query the desktop to check observable workspace/window/audio state.

The launch receipt confirms process creation, not that a graphical window has
finished loading. A move binds its target before execution and a move preview
checks for a focus change before confirmation. Later actions keep that target
even if a preceding workspace action changes focus.
Batch execution is not transactional, and previous effects are not rolled back
when a later action fails. Preview dismissal is available before execution;
in-flight cancellation and undo are roadmap work.

App launches go through `systemd-run --user` with type `exec` and literal
argument arrays. The user manager owns each app in a separate transient unit,
so restarting the assistant does not kill applications it launched. Display,
session, locale, and selected toolkit environment variables are forwarded by
name; shell/environment expansion of command arguments is disabled.

## Voice

A dedicated blocking worker owns the native speech objects. The async reader
pushes 20 ms PCM frames into an eight-frame queue, with backpressure. Only the
small keyword model decodes while waiting for a wake phrase. ASR decodes after
activation; dictation bypasses the keyword gate and never reaches the executor.

Native model inference does not run on the UI thread. Piper speech output is
cached; a separate eSpeak process performs phonemization. Capture is suspended
from recognition during spoken feedback, avoiding immediate self-triggering.
See [voice](voice.md) for precise limitations.

## Trust boundary

The protocol constrains what Hyprlume can do on behalf of a transcript/model.
It is not a sandbox against arbitrary programs running as the same user.
The daemon and its allowed launchers retain that user's normal desktop rights.
There is no authentication claim based on the wake word.

The local LLM adapter sends no screenshots or arbitrary file data, accepts no
redirects or non-loopback endpoints, and bounds its response. A loopback model
server must still be treated as a separately administered local program.

## Clipboard and capture

`services/clipboard` owns an opt-in wl-paste watcher, generation-bound delivery
and a bounded zeroizing text store. `services/capture` owns one cancellable
selector/screenshot/recorder job, validates geometry, creates private output
files and finalizes recordings with SIGINT. They publish independent opt-in
socket topics. `modules/tools` presents those snapshots without clipboard data
entering the AI state. Tool processes use literal argv; no shell is involved.
See [clipboard/capture behavior](clipboard-capture.md) for limits and qualification.

## Settings

`config/preferences` defines bounded appearance, shortcut and startup values.
`services/settings` publishes a separate settings topic, edits only validated
values, preserves app argv, detects stale/external edits and performs private
atomic writes on a blocking worker. The shell applies appearance tokens live;
voice/model and service defaults are applied on the next explicit daemon start.
`modules/settings` holds a local draft and matches acknowledgments by request
ID. An unrelated update cannot approve a pending save or retarget an agent plan.

## Locking

`services/locking` supervises a dedicated systemd notify unit and maintained
Swaylock client. Readiness follows the compositor acknowledgment; authentication
is never transported through shell/agent IPC. A private episode marker and
D-Bus unit observation survive daemon restarts. `app/locking` coordinates service
pausing, and command epochs prevent stale confirmations, late model plans or
post-lock playback. Native session tests use a new disposable CI user and
headless compositor, never the installed desktop. See [locking](locking.md).
