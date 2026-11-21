# Shell source layout

```
shell/
├── shell.qml                 services and live/ordinary-window host selection
├── DesktopShell.qml          per-output wallpaper/bar, focused home, command layer
├── PreviewWindow.qml         ordinary-window host for the live desktop
├── design-preview.qml        explicitly labelled sample-data preview / QA entry
├── ipc-test.qml              actual daemon subscription smoke entry
├── assets/
│   ├── backgrounds/          default wallpaper and lock background
│   └── shaders/              GLSL sources, portable bytecode and hash manifest
├── components/               shared materials, controls, icons and charts
├── models/                   derived presentation state, without IPC/processes
├── modules/
│   ├── scene/                wallpaper and background surface
│   ├── home/                 Overview and Quick Settings
│   ├── topbar/               per-output status bar and entry points
│   ├── command/              panel composition, editor, plan review and overlay
│   ├── connectivity/         Wi-Fi, Bluetooth and modal pairing prompts
│   ├── notifications/        centre, shared notice cards and non-focusing popups
│   ├── audio/                input/output/stream controls and selected media player
│   ├── power/                battery, brightness, profiles and session review
│   └── tray/                 application icons and native menu presentation
├── services/                 daemon socket, native audio, metrics, clock, workspace state
├── fixtures/                 synthetic data and UI-control probe; QA only
└── theme/Theme.qml           shared palette, typography and interaction timing
```

Visual modules receive small service interfaces as required properties. This
lets the **same** widget tree run with live services or explicit test fixtures.
Services own IPC and processes. `DesktopMenu.qml` presents the same sheets in both window types.
The shared appearance tokens are in `theme`, and provenance is in
[the asset catalog](assets/README.md).

`models/CommandState.qml` derives busy, review and voice state for
the Overview editor and command panel. `CommandEntry.qml` owns
local input/focus and emits submitted text; `PlanReview.qml` displays actions
and emits confirm/dismiss signals. `CommandPanel.qml` composes those pieces
and maps their intent to the existing protocol. None of the editor/review
components opens a socket or starts a process. The native input suite exercises
these exact components in both effects profiles.

`modules/connectivity/` follows the same boundary: its panes own selection and
temporary input, emit action objects, and never access D-Bus or start processes.
`Backend.qml` subscribes to the independent connectivity topic alongside agent
state. Pairing is a modal child with a single-use token. `PreviewData.qml`
supplies explicit fake wireless records; `scripts/qa/cases/connectivity.py`
drives these same controls with real pointer/keyboard input.

`qs.*` imports are resolved by Quickshell relative to the entry's shell root.
Keep entry files at that root. The AUR package excludes fixtures and QA entries.
The old separate plain-Qt palette was removed: tests now exercise the actual
command component, so a passing test cannot accidentally cover a different UI.

`services/AudioController.qml` validates native object handles and player
capabilities. `NativeAudio.qml` connects it to Quickshell's existing event-driven
adapters; `fixtures/AudioFixture.qml` supplies the same interface in tests. Audio
views use neither a separate Rust PipeWire client nor polling subprocesses.

`modules/power/` consumes the independent Rust power topic. It renders only
advertised capabilities and sends typed brightness/profile or prepare/confirm
requests. The session review has its own token; it never sends an agent command.
`fixtures/PowerFixture.qml` drives the same views without a native power API.

Run `python3 scripts/check-qml.py shell` for static checks. Rendering, input QA,
scaling, layers and reduced effects are documented in
[the desktop guide](../docs/homescreen.md) and [UI design record](../docs/design/shell-ui.md).
