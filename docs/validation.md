# Validation

Initial checks on 16 November 2026 used Arch Linux x86_64 on a Ryzen 5 5600H,
about 15 GiB usable RAM, Quickshell 0.3.2, and Qt 6.12. Native speech used CPU
inference. The installed desktop was left untouched; microphone capture and
physical speaker playback were not used for validation.

## Functional evidence

- Rust command, local-planner, private-socket, and HyprPilot protocol tests.
  These exercise previews, single-use confirmation, allowlists, failed
  postconditions, unknown model actions, and bounded responses.
- Four Qt Quick interaction cases, plus setup/cleanup, under Qt's offscreen
  software backend: Enter previews, explicit Run, confirmation, and busy state.
- An actual Quickshell `Backend.qml` connection to a simulated Rust daemon,
  including subscription and pushed action-review state (`shell/ipc-test.qml`).
- Native Qt Quick panel rendered and visually inspected (`assets/preview.png`).
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
QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
  qmltestrunner -input tests/qml -o -,txt
python3 scripts/benchmark.py
```

After explicitly installing the models and `espeak-ng`:

```sh
cargo test --release --locked --test voice_models -- --ignored --nocapture
python3 scripts/voice-smoke.py
```

The second command supplies synthetic PCM through a test capture executable;
it does not use the microphone or apply a desktop action.

## Still requires real-session testing

Physical microphone routing, false-wake rates, noisy rooms, accents, sleep and
device reconnects, simultaneous model workloads, Hyprland/NVIDIA rendering,
multi-monitor placement, and fractional scaling. aarch64 native archives are
supported by upstream and described in the recipe, but this release has only
been built and exercised on Linux x86_64 and development-tested on macOS arm64.
