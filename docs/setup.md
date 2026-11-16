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
systemctl --user import-environment WAYLAND_DISPLAY HYPRLAND_INSTANCE_SIGNATURE XDG_CURRENT_DESKTOP
systemctl --user start hyprlume-daemon.service hyprlume-shell.service
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

## Blur and keyboard shortcuts

The Wayland layer namespace is `hyprlume`. On current Lua-based Hyprland,
this opt-in rule enables compositor blur for these surfaces:

```lua
hl.layer_rule({ match = { namespace = "^hyprlume$" }, blur = true, ignore_alpha = 0.1 })
```

Refer to the matching version of the
[Hyprland layer-rule documentation](https://wiki.hypr.land/configuring/core/rules/layer-rules/)
for your installed compositor. Older `.conf` syntax differs.
Keep blur size/passes modest; it shares GPU resources with other work.

Bind a shortcut of your choice to `hyprlume toggle`, and a separate one to
`hyprlume voice off`. The pill is centered at the top with no reserved area;
move existing bars/dynamic islands or adjust `DesktopShell.qml` if they overlap.

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
