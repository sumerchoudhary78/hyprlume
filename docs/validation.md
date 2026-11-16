# Validation

Initial checks on 16 November 2026 used Arch Linux x86_64 on a Ryzen 5 5600H,
about 15 GiB usable RAM, Quickshell 0.3.2, and Qt 6.12. Native speech used CPU
inference. The installed desktop was left untouched; microphone capture and
physical speaker playback were not used for validation.

## Functional evidence

- 18 Rust command, local-planner, private-socket, and HyprPilot protocol tests.
  These exercise previews, single-use confirmation, allowlists, failed
  postconditions, unknown model actions, and bounded responses.
- Four Qt Quick interaction cases, plus setup/cleanup, under Qt's offscreen
  software backend: Enter previews, explicit Run, confirmation, and busy state.
- An actual Quickshell `services/Backend.qml` connection to a simulated Rust daemon,
  including subscription and pushed action-review state (`shell/ipc-test.qml`).
- Native Qt Quick panel rendered and visually inspected (`assets/preview.png`).
- Static QML checks (`python3 scripts/check-qml.py shell`) pass for the module
  tree. These check imports, bracket balance, and duplicate ids only; they do not
  parse or render QML.
- The desktop dashboard surfaces (`hyprlume-bar`, `hyprlume-home`) were added
  after this validation run. They have not been rendered on a live Hyprland
  session, and no frame-time or multi-monitor claim is made for them.
- The enabled user-service dependency graph is verified independently with
  `systemd-analyze`, without starting the graphical session or its services.
- An actual systemd user-manager test launches a non-interactive process from
  the daemon, stops the daemon, and verifies the app remains active in a
  separate control group; the test process is then stopped explicitly.
- Opt-in speech-model test: Piper-generated English → streaming ASR → typed
  workspace action; positive “Hey Pilot” detection and a negative wake example.
  This synthetic test is not a physical microphone/noise benchmark.
- Local model-server behavior tested with deterministic HTTP fixtures. No
  specific decision model's accuracy or latency is claimed.

HyprPilot tests use an isolated mock compositor socket. They verify the actual
typed dependency's wire requests and checked responses, without changing any
live windows or claiming real compositor qualification.

## Preliminary overhead measurement

`python3 scripts/benchmark.py` runs a release-build demo daemon and the actual
Quickshell preview UI under Qt's offscreen software renderer. Voice is off.
One run produced:

| Measurement | Result |
| --- | --- |
| Status request/response, median | 0.0214 ms |
| Status request/response, p95 | 0.0234 ms |
| Status request/response, p99 | 0.0335 ms |
| Rust daemon RSS | 23.98 MiB |
| Quickshell UI RSS | 121.79 MiB |
| Idle CPU ticks during 3-second sample | No ticks observed for either process |

The request sample contains 500 responses after 50 warmups. RSS includes the
process's resident shared pages, not just unique memory. The short CPU sample
has scheduler-tick granularity; it does not establish literally zero CPU use.

These measurements establish low idle overhead for this test configuration.
They do **not** measure GUI input latency, display frame times, compositor blur,
GPU power use, active speech cost, or LLM response time. UI feedback under
50 ms and idle shell CPU under 1% remain live-session qualification targets.

## Reproduce

```sh
cargo test --release --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/check-qml.py shell
QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
  qmltestrunner -input tests/qml -o -,txt
python3 scripts/benchmark.py
python3 scripts/check-units.py
```

After explicitly installing the models and `espeak-ng`:

```sh
cargo test --release --locked --test voice_models -- --ignored --nocapture
python3 scripts/voice-smoke.py
```

The second command supplies synthetic PCM through a test capture executable;
it does not use the microphone or apply a desktop action.

## Packaging

The AUR recipe disables makepkg's GCC LTO for native dependencies, following
[Arch's Rust packaging guidance](https://wiki.archlinux.org/title/Rust_package_guidelines).
Cargo's release profile retains Rust thin LTO. Dependencies are fetched during
`prepare`; build and package tests use `--frozen`.

## Desktop shell qualification

The dashboard has not been through a live session yet. The pass conditions are:

1. `hyprlume shell --path ./shell` starts with no QML errors; the bar appears on
   each monitor and windows tile below its reserved strip.
2. Switching to an empty workspace maps `hyprlume-home`; opening a window unmaps
   it. Switching workspaces with windows already open must not leave a stale
   dashboard on screen.
3. A workspace click in the rail, flow, or action ring switches to that
   workspace on both classic `.conf` and Lua (Hyprland 0.55+) sessions; named
   workspaces are valid targets in both.
4. On two monitors, focus changes move the dashboard to the focused output and
   the bar stays on both.
5. With the layer rules applied, all three surfaces blur; without them, they
   render flat but readable.
6. `inter-font` and `ttf-jetbrains-mono` are installed, and no label clips at
   1920x1080, 2560x1440, or a 1366x768 laptop.
7. Container numbers match `docker ps`/`podman ps`; the system card shows `—`
   for GPU on unsupported hardware and the sampler does not exit.

Record results in this file; do not treat the static checks as a substitute.

## Still requires real-session testing

Physical microphone routing, false-wake rates, noisy rooms, accents, sleep and
device reconnects, simultaneous model workloads, Hyprland/NVIDIA rendering,
multi-monitor placement, and fractional scaling. aarch64 native archives are
supported by upstream and described in the recipe, but this release has only
been built and exercised on Linux x86_64 and development-tested on macOS arm64.
