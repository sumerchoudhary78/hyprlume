# Shell source layout

```
shell/
├── shell.qml                 services and live/ordinary-window host selection
├── DesktopShell.qml          per-output wallpaper/bar, focused home, command layer
├── PreviewWindow.qml         ordinary-window host for the same live dashboard
├── design-preview.qml        explicitly labelled sample-data preview / QA entry
├── ipc-test.qml              actual daemon subscription smoke entry
├── assets/
│   ├── backgrounds/          generated room plate
│   ├── mascot/               transparent assistant art
│   ├── data/                 attributed public-domain world geometry
│   └── shaders/              GLSL sources, portable bytecode and hash manifest
├── components/               shared glass, icons, gauges, buttons and charts
├── models/                   derived presentation state, without IPC/processes
├── modules/
│   ├── scene/                wallpaper, Qt Quick 3D view and geometry generation
│   ├── home/                 dashboard composition, console, cards and action deck
│   ├── topbar/               bar content and navigation primitives
│   ├── command/              panel composition, editor, plan review and overlay
│   ├── connectivity/         Wi-Fi, Bluetooth and modal pairing prompts
│   └── notifications/        centre, shared notice cards and non-focusing popups
├── services/                 daemon socket, metrics, clock, workspace state
├── fixtures/                 synthetic data and UI-control probe; QA only
└── theme/Theme.qml           shared palette, typography and interaction timing
```

Visual modules receive small service interfaces as required properties. This
lets the **same** widget tree run with live services or explicit test fixtures.
Services own IPC/processes; components own presentation. Geometry belongs in
`modules/scene/Geometry.js`, composition landmarks in `Dashboard.qml`, shared
style in `theme`, and asset provenance alongside assets.

`models/CommandState.qml` derives busy/review/voice state from a snapshot for
the console, agent, command graph and command panel. `CommandEntry.qml` owns
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

Run `python3 scripts/check-qml.py shell` for static checks. Rendering, input QA,
scaling, layers and reduced effects are documented in
[the desktop guide](../docs/homescreen.md).
