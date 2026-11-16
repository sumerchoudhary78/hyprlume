# Cinematic desktop shell

Hyprlume's desktop follows the observatory reference: a planetary backdrop,
asymmetric floating glass cards, a central console, a right-side assistant,
and a holographic action deck. The screenshot in the README is rendered by
Quickshell, not a painted UI mockup.

## Rendering and depth

The room is a generated, cached wallpaper plate. Text, controls, gauges, charts,
workspaces, the command graph and the launcher are QML items. The robot is a
transparent, shaded image. The globe and city are real **Qt Quick 3D** geometry:
coastlines, land points, latitude/longitude lines and shaded building meshes.
Moving over the globe changes its orientation; clicking the globe or city
selects the network or container view.

`GlassSurface.qml` captures only the scene's wallpaper behind each card at half
resolution, freezes that capture between geometry/image changes, and applies a
precompiled optical shader. The rounded bevel displaces the image near its edge;
soft blur, slight colour dispersion, Fresnel highlights and a pointer light give
the surface thickness. A separate cached layer keeps foreground text and moving
water from re-blurring the glass. It never samples application windows or screenshots. Compositor
blur is optional for the small top-bar and command surfaces; no full-screen
Hyprland blur is needed for the dashboard.

The terrain is a perspective-projected mesh painted once per resize. The 3D
viewport has no continuous rotation or frame timer. Wave bars illustrate voice
state; they are not a measured microphone level. Motion is limited to hover,
voice-active feedback and actual state changes.

The system gauges use another small fragment shader: two wave layers, a bright
meniscus, depth shading, curved vessel reflections and a restrained caustic.
The **water area** tracks the metric percentage: a circular-segment calculation
converts each reading into height once, outside the shader. Empty and full
readings have no waves; unavailable/non-finite readings leave a dry vessel.
Labels are separate native text with a dark outline.

`LiquidMotion.qml` supplies one shared 62 ms timer for all three gauges.
Reading changes or hover wake it; unchanged readings settle after 3.2 seconds.
The timer stops when the dashboard is hidden, compact, covered by the command
overlay, or using reduced effects. Value easing uses this same clock. Glass
lighting changes only with pointer/style/geometry changes, with no time loop.

## Window ownership

| Namespace | Layer | Lifetime |
| --- | --- | --- |
| `hyprlume-wallpaper` | Background | One per output; empty input region |
| `hyprlume-bar` | Top | One per output; reserves its visible height |
| `hyprlume-home` | Bottom | Focused output, only on an empty workspace |
| `hyprlume-command` | Overlay | Only while a command panel is open |

Wallpaper remains behind ordinary app windows. The dashboard unmaps when an
app opens and returns when the workspace becomes empty. Metrics sampling stops
while the live dashboard is hidden. The clock wakes on minute boundaries.
`PreviewWindow.qml` uses the same dashboard and command components in an ordinary
window; `design-preview.qml` explicitly supplies synthetic data and labels it.

## Functional controls

- Quick actions open the command palette or terminal preview, change workspace,
  enable the wake word, and start dictation.
- The console and assistant accept real requests through the Rust daemon.
  Enter previews. The separate Run direct button explicitly requests execution.
- Review locks the request field. Confirm sends the current token; Dismiss
  cancels the pending preview. A busy planner disables replacement commands.
- The three graph nodes represent intent, review and execution. Execute is
  enabled only for an existing reviewed plan.
- Workspace rows use real Hyprland workspaces. System and runtime cards show
  host/PipeWire/container data; absent GPU/runtime information remains absent.
- The spatial deck opens terminal previews, command search, voice and workspace
  actions. The centre is a launcher, not an always-running animation.

Configure `[apps].terminal` in `~/.config/hyprlume/config.toml` before using the
terminal shortcuts, for example `terminal = ["foot"]`. The centre console is
Hyprlume's desktop-command interface; a full PTY runs in the configured terminal.
Code execution, deployments, cloud inventories and a general-purpose coding
agent are not implied by the reference's decorative labels.

## Layout and effects

Spatial landmarks are grouped in `modules/home/Dashboard.qml`, on a 1680×945
design canvas. It fits the available screen without cropping cards. At widths
below 1100 or heights below 620, a compact 1000×680 composition keeps the console,
rail and navigation, removing the side cards and holograms. Command overlays keep
readable independent dimensions.

Use `HYPRLUME_REDUCED_EFFECTS=1 hyprlume shell` for static water fills and to skip
the 3D viewport, glass refraction/blur and rim bloom. The backdrop and functional controls remain. This is also the
profile used by the CPU-software benchmark; it does not characterize GPU mode.

## Reproduce the design checks

On Arch, install `quickshell`, `qt6-quick3d`, `xorg-server-xvfb`, `xorg-xauth`,
`dbus`, `xdotool`, `python-pillow`, `inter-font` and `ttf-jetbrains-mono`. Then:

```sh
python3 scripts/check-qml.py shell
python3 scripts/build-shaders.py --check
python3 scripts/qa/check-effects.py
python3 scripts/qa/render-desktop.py --width 1920 --height 1080
python3 scripts/qa/render-desktop.py --interact
python3 scripts/qa/render-desktop.py --width 960 --height 640 --output output/design/compact.png
python3 scripts/qa/render-desktop.py --reduced-effects --output output/design/reduced.png
```

These use a private Xvfb display, private runtime directory and private D-Bus
session. Input tests exercise the actual widgets with pointer and keyboard
input. Fixtures never open a microphone, dispatch to a compositor, or call an
LLM. GitHub CI runs the same checks on Arch and uploads the rendered images.
Physical Wayland checks and remaining limits are recorded in [validation](validation.md).

`shell/assets/shaders/` holds GLSL sources, portable `.qsb` binaries and a hash
manifest. `scripts/build-shaders.py` bakes them offline using `qt6-shadertools`;
no compilation tool or source translation runs in the shell at startup. The
effects check reads actual mint pixels for empty, 18%, 50%, full and unknown
vessels, with a 3.5 percentage-point tolerance for the visible wave/meniscus.
It also tests hidden/reduced/restore/settled clock transitions.

[Asset origins and licensing](../shell/assets/README.md) ·
[Generation prompts](design/asset-prompts.md)
