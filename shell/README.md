# Shell sources

QML for the Hyprlume desktop shell. Entry point is `shell.qml`; Hyprlume runs it
with `qs --path shell/shell.qml` (or `qs --path shell/`).

```
shell/
├── shell.qml            entry point: services, then the desktop or preview root
├── DesktopShell.qml     composition root for layer-shell surfaces
├── PreviewWindow.qml    ordinary-window stand-in used by `--preview`
├── GlassPanel.qml       command panel (also the QML-test fixture)
├── LumeButton.qml       button used by the command panel
├── components/          reusable visuals, no Hyprland or protocol knowledge
├── modules/
│   ├── topbar/          always-on bar, one per monitor
│   └── home/            empty-workspace dashboard: stage, panels, rail, ring
├── services/            daemon link, clock, metrics, workspaces, shared state
├── services/system-metrics.sh   the one child process the shell owns
└── theme/Theme.qml      every colour, size, font and duration (`pragma Singleton`)
```

## Dependency rule

```
modules/*  →  components/*, services/*, theme/*
components/*  →  theme/*
services/*  →  daemon protocol, Quickshell services, theme (rarely)
theme/*  →  nothing
```

`modules` may import `components` and `services`. `components` must never
import `modules`, and nothing below `modules` may reach into a specific screen.
A component that needs shell-wide state takes it as a required property, which
is what keeps every surface previewable in isolation.

Imports use Quickshell's `qs.` prefix (`import qs.components`), so files can be
moved between directories without rewriting relative paths. `tests/qml` is the
exception: it uses plain Qt imports because it runs under `qmltestrunner` without
a Quickshell path, which is why `GlassPanel.qml` and `LumeButton.qml` stay at the
shell root.

## Adding a surface

1. Subclass `components/ShellWindow.qml` and set `surfaceName`. The window sets
   `WlrLayershell.namespace: "hyprlume-<surfaceName>"`; that exact string is what
   the compositor layer rule matches for blur. Do not hard-code the namespace in
   the module.
2. Choose the layer deliberately: `WlrLayer.Bottom` for wallpaper-adjacent
   content, `Top` for bars, `Overlay` for anything that takes keyboard focus.
3. Give it an `exclusiveZone` if Hyprland should reserve space, otherwise `0`.
4. Bind visibility to real state (`Workspaces.desktopVisible`, `Backend.online`)
   so an unused surface unmaps instead of painting.
5. Add the namespace to the layer rules in `docs/setup.md`.

## Responsiveness

Layouts switch on logical-pixel breakpoints, not fluid scaling, so text never
lands between two sizes. See `modules/home/Home.qml` (`showRail`, `showLeft`,
`showRight`, `showBand`) and `modules/topbar/TopBar.qml` (`visibleModes`,
`showModeLabels`). `Theme.px()` applies the one user scale preference; monitor
scale is already handled by Hyprland.

## Checks

```sh
python3 scripts/check-qml.py shell
```

That covers bracket balance, `qs.*` imports that do not resolve, and duplicate
`id`s. It is not a parser and does not replace `qmllint` or a live session; it
catches the errors that would otherwise only appear when the shell reloads.
