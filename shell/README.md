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
│   └── data/                 attributed public-domain world geometry
├── components/               shared glass, icons, gauges, buttons and charts
├── modules/
│   ├── scene/                wallpaper, Qt Quick 3D view and geometry generation
│   ├── home/                 dashboard composition, console, cards and action deck
│   ├── topbar/               bar content and navigation primitives
│   └── command/              the single command panel and overlay implementation
├── services/                 daemon socket, metrics, clock, workspace state
├── fixtures/                 synthetic data and UI-control probe; QA only
└── theme/Theme.qml           shared palette, typography and interaction timing
```

Visual modules receive small service interfaces as required properties. This
lets the **same** widget tree run with live services or explicit test fixtures.
Services own IPC/processes; components own presentation. Geometry belongs in
`modules/scene/Geometry.js`, composition landmarks in `Dashboard.qml`, shared
style in `theme`, and asset provenance alongside assets.

`qs.*` imports are resolved by Quickshell relative to the entry's shell root.
Keep entry files at that root. The AUR package excludes fixtures and QA entries.
The old separate plain-Qt palette was removed: tests now exercise the actual
command component, so a passing test cannot accidentally cover a different UI.

Run `python3 scripts/check-qml.py shell` for static checks. Rendering, input QA,
scaling, layers and reduced effects are documented in
[the desktop guide](../docs/homescreen.md).
