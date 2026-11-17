# Validation

Checked on **16 November 2026**, on Arch Linux x86_64, Ryzen 5 5600H,
approximately 15 GiB usable RAM, Quickshell 0.3.2, Qt 6.12 and Hyprland 0.57.1
with a Lua configuration. The physical output was 1920×1080 in its 144 Hz mode.
No physical microphone recording or speaker playback was used.

The source refactor and Wi-Fi/Bluetooth service were checked on **17 November
2026** with the native Arch runtime and macOS arm64 development builds. The older physical-session and overhead
samples below describe the desktop releases measured on 16 November.

## Automated checks

- **39 Rust checks** (28 integration tests and 11 unit tests) cover bounded commands, reviewed local model
  plans, single-use confirmations, private sockets, single-instance ownership,
  allowlists, classic/Lua dispatch and checked desktop postconditions.
- Voice-event tests protect pending reviews and running commands from later
  wake/partial/final events, and ensure dictation remains text. Socket tests
  cover invalid/oversized frames and clients that have not subscribed.
- Wireless tests cover stale targets, credential/debug/error redaction, bounded
  snapshots, concurrent-operation rejection, discovery cleanup, cancellation
  before a prompt, expiring/single-use pairing tokens and independent socket
  topics. Wireless requests preserve an AI command's pending confirmation.
  The command shortcut brings a preview arriving behind Connections forward
  with one press, preserving its token.
- The complete core suite also passes without the optional voice feature;
  that configuration is exercised in CI.
- The Lua tests include bound-window moves, unique focus, malformed provider
  metadata and an address that must never become executable Lua.
- Native Quickshell/RHI rendering passes at 1920×1080, at 960×640 in the compact
  composition, and with reduced effects. Exported images are visually inspected.
- The liquid material fixture checks actual water pixels for 0%, 18%, 50%, 100%,
  unavailable and non-finite readings. Measured areas were 0%, 19.52%, 51.64%,
  100%, 0% and 0%; moving surfaces/menisci allow 3.5 percentage points. The shared
  motion clock passes hidden, reduced, resume and idle-settle checks. GLSL is
  baked offline and source/binary hashes are verified before rendering.
- Real pointer/keyboard input drives the actual dashboard and command components
  on a private Xvfb display. Checks cover Enter-to-preview, busy controls, review
  locking, explicit Run direct, confirmation tokens, dismissal, wake/off mode
  requests, workspace rows, navigation, 3D globe/city actions and the launcher.
- Wireless pointer/keyboard fixtures exercise scans, busy controls, password
  submission, device-bound forgetting, matching-code confirmation and PIN entry
  during unrelated service updates, with redacted logs. Normal,
  absent-adapter and pairing/PIN layouts render through the actual QML/RHI path,
  including a 700×680 tabbed panel.
- Fixtures are clearly labelled and never access a microphone, real desktop,
  local model server or cloud provider. Production packaging excludes them.
- Static QML checks validate import directories, delimiters and duplicate ids.
  They complement runtime tests; they are not a QML parser.
- The enabled service dependency graph is checked with `systemd-analyze`.
  A previous systemd test also verified independent app lifetimes across daemon
  restarts; the short-lived test process was explicitly stopped afterward.

The opt-in synthetic English Piper → ASR → typed-action and wake-word fixtures,
including the injected-PCM daemon flow, passed again after the source refactor.
The injected flow produced a workspace-2 preview and stopped capture without
executing a desktop action. Speech recognition
quality, false wakes and real devices require separate physical testing; see
[voice](voice.md). No decision model's planning quality is claimed from the HTTP
protocol fixtures.

## Physical session checks

After the user requested replacement, the new shell and real Rust daemon were
installed as user services in the active account. The previous shell/test daemon
was stopped. The prior startup configuration was backed up before changes.

The following passed against the real compositor:

1. Wallpaper is Background with an empty input region, dashboard is Bottom,
   and the bar is Top with a 72-pixel reservation at 1920×1080.
2. Opening a temporary Foot test window hides the dashboard while the wallpaper
   and bar remain. Closing that specific test window restores the dashboard.
3. Opening/closing the command palette maps/unmaps its Overlay surface.
4. Rust commands switch to workspace 10 and restore workspace 1, with observed
   postconditions, using the actual Lua config provider.
5. The temporary verification window is focused by its unique app id, moved
   to workspace 10 and back to 1, and then closed. Other windows are not targeted.
6. The graphical configuration parses, both user services remain active, and
   `hyprlume doctor` finds the voice models and required tools. Voice stays off.

These checks do not establish full desktop-environment compatibility, every
keybinding, lock-screen integration, notification handling or multiple outputs.
The test window is deliberately temporary; no user's application was closed.

## v0.2.0 overhead sample

A 20-second sample after initialization used the visible full-effects desktop,
real host telemetry every two seconds, GPU readings at up to ten-second intervals,
and microphone off. Process-tree CPU includes sampler subprocesses and is a
percentage of **one core**.

| Measurement | Observed |
| --- | --- |
| Rust daemon process-tree CPU | 0.0% in the short sample |
| Shell process-tree CPU | 0.6% of one core |
| Rust daemon RSS | 23.24 MiB |
| Shell RSS, full 3D profile | 620.41 MiB |

RSS includes mapped/shared graphics-runtime pages; this is not unique memory or
VRAM. The full 3D profile has a material memory cost. The lower-effects profile
skips 3D, glass blur and rim bloom; its memory benefit has not been quantified
here. Graphics resources are retained while the home surface is unmapped to
allow fast return, while host sampling pauses.

These are one-machine observations, not guarantees. The display's 144 Hz mode
is **not** a measured application frame rate. CPU sampling has scheduler-tick
granularity; 0.0% does not establish literally no work. Input latency, frame
pacing, GPU power, concurrent speech and LLM cost still need profiling. The
under-50-ms input-feedback goal remains a measurement target.

## v0.3.0 liquid-effects sample

On the same physical session, separate 20-second service-cgroup samples with
microphone off measured the following. The full dashboard was visible with
real telemetry and liquid motion enabled. A temporary verification window then
hid the home surface; closing only that window restored it. Glass and liquid
rendering were also visually checked on the real Wayland output.

| Measurement | Visible dashboard | Home hidden |
| --- | --- | --- |
| Rust daemon CPU, one core | 0.000% in sample | 0.000% in sample |
| Shell CPU, one core | 3.864% | 0.002% |
| Rust daemon RSS | 25.46 MiB | 25.46 MiB |
| Shell RSS | 565.65 MiB | 536.86 MiB |

These use cgroup `usage_usec`, including short-lived sampler children. The
older tick-based sample above is not a controlled comparison. Liquid motion
has a visible-session cost; idle below 1% is a target, not achieved in this
sample with changing metrics. Hiding the dashboard stops its water clock and
telemetry. Graphics resources are retained for return; RSS varies with runtime
allocations and includes shared mappings. GPU power and frame pacing were not
measured. Raw conditions/results: [performance-liquid.json](design/performance-liquid.json).

## Reproduce

```sh
cargo test --release --locked --all-targets
cargo test --locked --no-default-features --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/check-qml.py shell
python3 scripts/build-shaders.py --check
python3 scripts/qa/check-effects.py
python3 scripts/qa/render-desktop.py --width 1920 --height 1080
python3 scripts/qa/render-desktop.py --interact
python3 scripts/qa/render-desktop.py --width 960 --height 640 --output output/design/compact.png
python3 scripts/qa/render-desktop.py --reduced-effects --output output/design/reduced.png
python3 scripts/check-units.py --binary target/release/hyprlume
```

For a read-only sample of an already running systemd user session:

```sh
python3 scripts/qa/profile-live.py --seconds 20 --label 'visible full effects; microphone off'
```

This reads cgroup CPU usage (including short-lived sampler children) and the
RSS of current processes. Record the actual visibility, effects and voice state
in the label; the profiler does not change the desktop or audio mode.

`scripts/benchmark.py` measures a simulated daemon and the reduced-effects
ordinary-window dashboard with Qt's CPU software backend. It does not measure
the full-effects GPU profile above. Its output describes its own conditions.

After explicitly installing the English models and `espeak-ng`:

```sh
cargo test --release --locked --test voice_models -- --ignored --nocapture
python3 scripts/voice-smoke.py
```

The second voice command supplies synthetic PCM through a capture fixture.
Neither is a physical microphone benchmark.

## Wireless qualification

On 17 November, a separate native daemon read system D-Bus on the Arch host.
Its snapshot found the connected Wi-Fi interface, observed the active network,
matched NetworkManager's enabled state and found the powered Bluetooth adapter.
No scan, radio toggle, connection handoff, forget or pairing action was sent to
real hardware. Live network identifiers are excluded from public fixtures.

Physical Wi-Fi credential failures/retries, driver handoff, Bluetooth pairing
methods/devices, authorization policies and service restart/recovery still need
hardware qualification. Connection and pairing UI tests use synthetic data;
they verify intent, state and protocol behavior rather than a physical radio.
Supported modes and current profile-editor limits are in
[connectivity](connectivity.md).

## Notifications (unreleased)

The notification service is tested with private D-Bus daemons and synthetic
QML clients. The Linux suite includes real notification methods, owner-directed
action/close signals and a replaceable competing server that Hyprlume leaves
in place. Releasing that server's name allows Hyprlume to acquire it. A caller
cannot bypass unavailable ownership by targeting the pre-registered unique
connection. Pure lifecycle tests cover expiry, replacement, sender isolation,
stale action revisions, resident/critical/transient hints, DND, lost-bus state,
one-time close reasons, history bounds and reply frame size.

Native input checks cover opening the centre, DND, actions, history removal,
clear and Escape. Centre, popup and compact reduced-effects renders use the
same production QML components. These run on a private Xvfb display; they do
not replace the active session's notification daemon. No live-session upgrade
or notification-server takeover was performed. Layer-shell popup keyboard
policy is explicitly `None`; a physical Wayland notification-client matrix
remains to be qualified. See [notification limits](notifications.md).

## Audio/media (unreleased)

On 17 November, the native audio adapter passed against a private software-only
PipeWire server and fake MPRIS player. Volume, mute, playback and metadata made
real protocol round trips; independent server queries observed the requested
volume/mute and configured default. Controller checks rejected invalid volumes,
unavailable/unbound/stale nodes, stale players and unsupported playback actions.
Native pointer/keyboard tests cover mute, volume, default selection, player
selection, capability disabling and Escape. Full, compact reduced-effects and
unavailable states render with the production components. Existing command and
notification input tests still pass. These checks never connect to the active
desktop's PipeWire socket or players. See [audio limits](audio.md) for the
remaining physical-device, policy and performance qualification.

## Power/session (unreleased)

Native power tests use owned private D-Bus services and temporary backlights.
They observe UPower state, profile/brightness read-back and exact logind method
arguments, including inhibitor flags. Remote/other-user/ambiguous/replaced
sessions, service loss, inhibitor changes and stale backlights are rejected.
Controller tests cover single-use confirmation, cancellation/expiry, closing
during reads, concurrency, bounded errors and late disconnected snapshots.
An idle regression checks that closed panels stop D-Bus polling and periodic
brightness observation. Socket coverage keeps the power topic independent and
preserves agent confirmations. Native pointer/keyboard checks exercise the
production panel and compact scrolling, unavailable/inhibited controls and
separate session review. These do not invoke a real host power method. Physical
hardware, policy, sleep/resume and logout teardown remain unqualified; see
[power details](power.md).

## Packaging and remaining qualification

Tray validation uses a private StatusNotifier/DBusMenu app and native Quickshell
adapters, checking icon loading and the exact activation/secondary/scroll/menu
events received by that app. Submenu lifetime, check-state updates, disabled
entries and stale handles are exercised. Pointer/keyboard fixtures use the same
production controls. See [tray limits](tray.md) for remaining app qualification.

The AUR recipe fetches during `prepare` and builds/tests with `--frozen`. GCC
LTO is disabled for native dependencies; Rust thin LTO remains enabled. The
runtime includes the whole QML module/asset tree, excluding QA fixtures.

Still open: long-running reliability, physical audio routing/noise/accents,
sleep/resume, simultaneous model workloads, NVIDIA/other-driver comparisons,
fractional scaling and multiple monitors, app-wide theming, notification/power/
lock integration, and aarch64 build/runtime qualification. The recipe lists the
upstream aarch64 archive, but this desktop release has been exercised on Linux
x86_64 and Rust-development-tested on macOS arm64.
