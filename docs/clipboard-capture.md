# Clipboard and screen capture

Open the tools button in the bar, `hyprlume clipboard` or `hyprlume capture`.
These surfaces share the glass theme and do not require the AI planner.

## Text clipboard history

History starts **off**. Enable it explicitly in the Clipboard tab. `wl-paste`
watches text offers without a polling loop and delivers bounded UTF-8 text
through the private Unix socket. Known `sensitive`, `nil` and `clear` offers are
discarded before reading stdin. Wayland clients do not universally identify
passwords: pause history before copying secrets. Oversize, binary and empty
offers are skipped. There is no image or primary-selection history.

The store retains 24 deduplicated items, at most 3 KiB each, in memory. Only a
160-character preview and an opaque ID enter its separate UI subscription; full
payloads never enter agent prompts, agent snapshots, telemetry or application
logs. Copy sends the selected current entry to `wl-copy` on stdin, with fixed
arguments. It does not type into or paste into another application. Remove and
Clear forget stored items; they do not change the current Wayland selection.
Pause stops new collection and keeps the existing list. Restarting the daemon
forgets history. Stored strings are cleared with `zeroize` when dropped; this
does not promise erasure of every OS/IPC buffer or of the originating app.

Watch generations revoke deliveries from a watcher that was paused, stopped or
replaced. Unexpected watcher exit pauses history and offers an explicit retry;
it does not spin in a restart loop. Install Arch's `wl-clipboard` package.

## Screenshots and recordings

- Screenshot captures the output layout with `grim`; Select area opens `slurp`.
- Record this display targets the monitor hosting the panel directly. Record area
  selects a rectangle. API clients without an output name fall back to selecting
  an output.
  Both use `wf-recorder`, fixed 30 fps capture and Matroska. They record
  **no microphone or desktop audio**.
  Fixed-frame capture keeps the recorder responsive to Stop even when a selected
  output is completely static; no recorder runs while idle.
- The overlay dismisses before capture. A persistent bar button appears while
  the recorder process is running; Stop sends SIGINT to that owned child and
  waits up to eight seconds for finalization. `hyprlume capture --stop` also
  works when the panel is closed. Cancel stops a pending selector or screenshot.
- The daemon serializes capture jobs and stops its owned job on shutdown.
  Failed/timed-out recordings can leave a partial MKV for recovery. A process
  starting is not proof that its compositor/codec produced frames; an empty
  file or unexpected exit is reported as an error.

Screenshots go to `~/Pictures/Hyprlume`, recordings to `~/Videos/Hyprlume`.
Absolute `XDG_PICTURES_DIR` and `XDG_VIDEOS_DIR` environment values override the
base directory. The service does not evaluate shell syntax in `user-dirs.dirs`.
Hyprlume creates an owned 0700 output directory and exclusive 0600 files. An
existing public or symlinked Hyprlume output directory is refused. Paths appear
in the Capture tab, and files are not uploaded or sent to a model. Multi-output
recording is deliberately an explicit single-display target.

Install `grim`, `slurp`, and `wf-recorder`. Compositor screencopy/layer-shell and
a usable encoder are required. The UI disables unavailable tools. Newly
installed tools are discovered on daemon restart.

The packaged session disables Hyprland's layer animation for the tools sheet
so its closing frame stays out of captures. If you run the shell in your own
Hyprland Lua profile, add the same rule to that profile.

```lua
hl.layer_rule({ match = { namespace = "^hyprlume-tools$" }, no_anim = true })
```

## Checks and qualification

`scripts/qa/check-desktop-tools.py --binary target/debug/hyprlume` starts the
actual daemon with private XDG directories and fixture executables at the front
of **only that child's** PATH. It verifies real watcher-to-socket delivery,
sensitive-offer rejection, stale generations, copy stdin, file modes, region
handling and graceful SIGINT finalization. No host clipboard or screen is read.
Rust tests cover limits, duplicates, revocation, geometry and capture admission.
`render-desktop.py --tools --interact` drives the production QML with synthetic
data and real pointer input. Full/compact screenshots are CI artifacts.

These tests establish orchestration and UI behavior. Real clipboard MIME
interoperability, screencopy permissions, fractional scaling and codec/GPU
behavior still require physical-session release qualification.
