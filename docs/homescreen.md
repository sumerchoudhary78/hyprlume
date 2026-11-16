# Home screen

The desktop shell is a Hyprland-native dashboard: a bar that is always there,
and a home surface that appears when the focused workspace is empty. It targets
the layered-glass look of the concept render in the design brief, with every
element bound to state Hyprlume actually has.

This document explains what maps to what, why the surfaces are layered the way
they are, what the compositor does versus what QML paints, and what was
deliberately left out.

## Surfaces

| Surface | Type | Layer | Visible when |
| --- | --- | --- | --- |
| `hyprlume-bar` | `PanelWindow`, one per screen | Top | Always |
| `hyprlume-home` | `PanelWindow`, focused screen | Bottom | Focused workspace has no windows |
| `hyprlume-command` | `PanelWindow` overlay | Overlay | `snapshot.panel_open` |

The home surface being `Bottom` and full-screen is the important decision. It is
not an overlay above windows, so it never intercepts a click meant for an
application, and it is not a second compositor feature: while the focused
workspace has no windows it maps, otherwise it unmaps and costs nothing. A
bottom-layer surface also lets Hyprland draw windows above it without special
rules. It does take pointer input while mapped, which is the point — the
dashboard is clickable — and means the desktop's own context menu is not
reachable from an empty workspace while the dashboard is visible.

The one bar per monitor uses `Variants` over `Quickshell.screens`, while the
dashboard explicitly belongs to `Hyprland.focusedMonitor` — mapping a dashboard
on every monitor would put four copies of the same CPU graph on screen.

## Reference mapping

The concept is a dashboard for a fictional deployment product. Panels with no
data behind them would be theatre, so the layout is kept and the contents are
replaced with real shell verbs.

| Reference element | Implementation | Data source |
| --- | --- | --- |
| Top bar, brand, search, mode chips | `modules/topbar/` | `ShellState.mode`, `Backend.snapshot` |
| Left quick-action card | `QuickActions.qml` | command open, workspace dispatch, voice modes |
| Build/test/deploy graph | `WorkspaceFlow.qml` | live Hyprland workspace list |
| Centre terminal stage | `Stage.qml` + `StageBackdrop.qml` | daemon snapshot: phase, transcript, receipts |
| Clock / weather card | `ClockCard.qml` | system clock, uptime, load |
| Assistant panel | `AgentPanel.qml` | daemon message, voice mode, prompt shortcuts |
| Right workspace list | `WorkspacesPanel.qml` | `Hyprland.workspaces` + toplevel counts |
| System gauges | `SystemPanel.qml` | `/proc`, sysfs |
| Container card | `ContainerPanel.qml` | `docker ps` / `podman ps`, if installed |
| Circular action ring | `ActionRing.qml` | command panel toggle, voice modes, next workspace |
| Left app-icon rail | `Rail.qml` | workspace tiles, one per workspace |

Deliberate omissions: weather (no data source), the notification bell (no
notification backend), and the app-icon dock (icons with no launch semantics are
decoration; workspace tiles are bound to workspaces instead). See
[roadmap](roadmap.md) for the surfaces that would need real backends first.

## What QML paints and what the compositor does

QML cannot sample the wallpaper behind a surface, so "real" glass refraction is
out of scope for a layer-shell client. The look is split in two:

- **QML** paints the tint gradient, rim, top sheen, and accent bloom
  (`components/GlassSurface.qml`). These are cheap rectangles and gradients,
  not shaders.
- **Hyprland** blurs whatever is behind the surface. The window only declares a
  namespace; the compositor rule decides blur. This keeps blur settings in the
  user's compositor config where they belong, and avoids a per-surface
  `BackgroundEffect` object that would double-blur when a layer rule exists.

Layer rules for the three namespaces are in [setup](setup.md).

Three options were evaluated for blur:

1. **Hyprland layer rule (used).** Works on any current Hyprland, no extra
   packages, controllable through `blur` size/passes. Limitation: it blurs the
   whole surface, not per-element, so the dashboard's inner cards share one
   backdrop.
2. **`ext-background-effect-v1`.** Per-surface background blur through a
   protocol Hyprland implements behind a flag on some builds. Cleaner blur
   regions, but support varies by version and by whether the protocol is
   exposed to layer-shell clients. Not enabled by default.
3. **`hyprglass`-style shader effects.** Closest to the reference (refraction,
   per-element distortion). Requires a custom plugin or patched compositor and
   is not something a package should assume. Documented as optional, not
   implemented.

The stage backdrop is procedural (`Canvas`, painted on resize only): horizon
glow, three ridgelines, deterministic stars. The reference's 3D scene is
wallpaper, not a render target — putting a live 3D scene inside a QML panel would
spend the whole frame budget on decoration.

## Responsiveness

Breakpoints are logical pixels at `Theme.scale = 1.0`; Hyprland applies monitor
scale, so a HiDPI screen gets the same layout, not a denser one.

| Width | Result |
| --- | --- |
| < 760 | Stage only; no rail, side columns, or bottom band |
| ≥ 760 | Bottom band (system, ring, containers) when height ≥ 720 |
| ≥ 880 | Workspace rail |
| ≥ 1180 | Left column (quick actions, workspace flow) |
| ≥ 1480 | Right column (agent, workspaces) |

Height is handled inside the columns rather than by dropping them: the
workspace flow and workspace list shrink their row counts, the agent panel hides
its prompt shortcuts below 380 px, quick actions drop rows below 286 px, and the
rail caps its tiles to the available height. Panels whose minimum content no
longer fits (`WorkspaceFlow` under 260 px, `WorkspacesPanel` under 320 px) are
hidden entirely, which gives the remaining card the space instead of clipping
it.

The bar degrades separately: labels below 1380, mode count 3 → 5 → 6 at 980 /
1240 / 1560, icon-only below 980. The search field shrinks before any control is
dropped. Panels that disappear are `visible: false`, not merely clipped, so
their repeaters and bindings stop doing work.

## Performance budget

The shell is idle most of the time and must stay that way.

- One child process total: `services/system-metrics.sh` at 1 Hz, emitting two
  compact lines. Container stats are sampled every fifth tick, with `timeout`
  guards so a hung runtime cannot stall the sampler.
- The home surface unmaps when a window exists; no repaint, no bindings on
  screen.
- The stage backdrop repaints on resize only. No shader effects, no looping
  animations, no frame timers.
- Gauges and sparklines are `Canvas`/`Shape` items redrawn when values change,
  not per frame.
- The daemon snapshot arrives over one persistent socket; the UI has no polling
  loop of its own.

Targets to verify on a real session: idle shell CPU under 1%, no frame drops
while the dashboard is mapped, and under 50 ms from click to visible response.
Those are targets, not measurements — this tree has not been run on a Hyprland
session yet.

## Verification status

Static checks pass (`python3 scripts/check-qml.py shell`). Runtime verification
still needs a Linux machine with Quickshell and Hyprland:

- `qs --path ./shell` loads and resolves `qs.*` imports.
- `ClippingRectangle` needs Qt ≥ 6.7 for the per-corner radius properties.
- `Hyprland.focusedMonitor`, `Hyprland.toplevels`, and `Hyprland.workspaces`
  property names match the installed Quickshell version.
- Fonts: install `inter-font` and `ttf-jetbrains-mono`; otherwise Qt substitutes
  and metrics shift.
