# Setup

## Your user session

Start the daemon and UI inside the same user's Hyprland session. The daemon
discovers Hyprland through `HYPRLAND_INSTANCE_SIGNATURE` and `XDG_RUNTIME_DIR`.
It does not attach to another account's desktop or request root access.

Use `hyprlume doctor` to check paths and required tools without recording audio.
If no Hyprland session is found, workspace/window actions fail with a clear
error; demo mode can still exercise the UI and protocol.

## Configuration

The optional configuration file is
`$XDG_CONFIG_HOME/hyprlume/config.toml` (normally
`~/.config/hyprlume/config.toml`). Omitted fields use defaults.
`hyprlume config` prints the effective configuration. Use `--config PATH` to
load a different file. Restart the daemon after changing configuration.

A small example:

```toml
[apps]
browser = ["firefox"]
terminal = ["kitty"]
files = ["nautilus"]

[voice]
execute_commands = false
speak_results = true
threads = 2
```

App names must be lowercase. Launchers are argument arrays, passed directly
to the executable without shell interpolation. Only configure programs you
trust. Then `open browser` resolves to your configured launcher.
Application launches use the systemd user manager in a separate transient unit;
restarting the assistant leaves those applications running. A working user bus
and `systemd-run` are required for app launch actions.

## Optional local decision model

Start a compatible `llama-server` separately, bound to loopback, with a model
that follows constrained JSON instructions. A quantized 3–4B instruction model
is a starting point for a small GPU; benchmark context length and GPU offload
on your hardware. Model file size is not the total VRAM requirement.

```toml
[llm]
endpoint = "http://127.0.0.1:8080/v1/chat/completions"
model = "local"
timeout_seconds = 20
```

The endpoint must be an HTTP loopback IP address. Redirects and proxy use are disabled.
Requests contain the user's command, allowed app names, and an action schema;
they do not include screenshots, audio, clipboard contents, or files.
Each generated plan is validated again in Rust and requires confirmation,
including when the original request used `--execute`.

This release does not download or supervise a decision model. Runtime support
for the JSON-schema request must be enabled by the model server. The adapter
is protocol-tested; model-specific planning quality must be evaluated separately.

## User services

Run these from a terminal **inside your graphical session**:

```sh
systemctl --user import-environment WAYLAND_DISPLAY HYPRLAND_INSTANCE_SIGNATURE XDG_CURRENT_DESKTOP XDG_SESSION_ID
systemctl --user restart hyprlume-daemon.service hyprlume-shell.service
```

Both units stop with `graphical-session.target`. If your session manager owns
that target and imports the graphical environment, you can enable the units:

```sh
systemctl --user enable hyprlume-daemon.service hyprlume-shell.service
```

For a bare Hyprland session, use its startup mechanism to import the environment
and start the services. Do not start a second shell/daemon manually at the same
time. Stop them with:

```sh
systemctl --user stop hyprlume-shell.service hyprlume-daemon.service
```

The daemon is a desktop controller running with your user's rights. Its
confirmation rules are not an operating-system sandbox. Launchers must be
able to write their own settings and cache, so the service does not impose a
read-only home on child applications.

The optional [Polkit agent](polkit.md) can supply system authorization prompts
in a matching development build. Its separate user service is enabled explicitly.

## Fonts

The shell asks for **Inter** and **JetBrains Mono**. On Arch:

```sh
sudo pacman -S inter-font ttf-jetbrains-mono
```

Without them Qt substitutes its default sans/mono, which changes text widths and
can overflow tightly fitted rows. No font files are bundled with the package.

## Blur, layer rules, and shortcuts

The shell uses four Wayland layer roles:

| Namespace | Surface |
| --- | --- |
| `hyprlume-wallpaper` | persistent background, one per monitor |
| `hyprlume-bar` | top bar, one per monitor |
| `hyprlume-home` | empty-workspace dashboard |
| `hyprlume-command` | command panel overlay |

The room wallpaper uses `hyprlume-wallpaper` on the Background layer with an
empty input region. It persists behind apps and does not reserve screen space.
The dashboard samples this same room asset inside QML for its own glass blur.
It does not need a full-screen compositor blur rule.

For optional blur behind the bar, command and Connections overlays on Lua Hyprland:

```lua
hl.layer_rule({ match = { namespace = "^hyprlume-(bar|command|connectivity)$" }, blur = true, ignore_alpha = 0.1 })
```

Refer to your version's [layer-rule documentation](https://wiki.hypr.land/configuring/core/rules/layer-rules/).
The bar reserves its actual scaled height (72 pixels on a 1920×1080 screen).
Replace an existing bar before starting this one to avoid stacking reservations.

Bind `hyprlume toggle` to your preferred launcher key and `hyprlume voice off`
to an easy microphone-off shortcut. For example, in a Lua config:

```lua
hl.bind("SUPER + K", hl.dsp.exec_cmd("hyprlume toggle"))
hl.bind("SUPER + SHIFT + M", hl.dsp.exec_cmd("hyprlume voice off"))
```

The package does not edit your Hyprland config. When starting user services
from a session startup hook, first import the graphical environment, then
**restart** both services; this avoids retaining an old instance signature from
a lingering user manager. A session manager that imports the environment before
activating `graphical-session.target` can manage the enabled units normally.

Workspace UI controls inspect `Hyprland.usingLua`. Rust actions read
`j/status.configProvider` through HyprPilot. Lua sessions receive bounded
`hl.dsp.*` expressions; classic sessions use the typed legacy dispatchers.
Relative UI navigation uses `+1` and `-1`, including on Lua sessions.

For lower graphics cost:

```sh
HYPRLUME_REDUCED_EFFECTS=1 hyprlume shell
```

That disables 3D, local glass blur and rim bloom. [The desktop guide](homescreen.md)
explains native previews, geometry, layouts and runtime data sources.

## Troubleshooting

- **Disconnected:** start the daemon; check the same user's runtime directory.
- **Another daemon is running:** use `systemctl --user status` or stop the
  existing process. Do not delete an active socket directory.
- **Models missing:** run `hyprlume-models` as your user, then `hyprlume doctor`.
- **Wake mode fails under SSH:** log into your own graphical session first;
  microphone devices are granted by the session's audio/device policy.
- **Speech output unavailable:** install `espeak-ng`; check `pw-play` and the
  default PipeWire output. Desktop actions remain completed even if TTS fails.
- **Focus is ambiguous:** more than one window has the requested app class.
  Focus it manually; this preview intentionally avoids guessing a target.
- **Preview expired:** request a fresh preview and confirm within 30 seconds.

## Unreleased desktop services

Current `main` adds the [launcher](launcher.md), [clipboard/capture](clipboard-capture.md),
[settings](settings.md) and [secure lock service](locking.md), alongside tray,
notifications, sound/media and power. These require a source build until a new
release is published; the v0.4.1 package remains unchanged. Optional Arch tools
are `wl-clipboard`, `grim`, `slurp`, `wf-recorder`, `swaylock` and `librsvg`.

Development visual checks use `shell/design-preview.qml` with synthetic data
and private displays. `hyprlume shell --preview` is an ordinary-window host for
the live backend when connected; it is not a sandbox for desktop actions. Use
the documented isolated QA scripts when preserving the running desktop.
