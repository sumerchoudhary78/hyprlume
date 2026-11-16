# Roadmap

The first release establishes a usable command/voice path. A full desktop shell
and an unrestricted computer-use agent are larger projects.

## Next: qualify the interactive path

- Physical-microphone tests for false wakes, accents, noisy rooms, silence,
  device disconnects, and sleep/resume.
- Live Hyprland/NVIDIA/fractional-scaling testing, keyboard focus, multi-monitor
  placement, and compositor frame-time measurements.
- Richer target selection and persistent undo/snapshot integration with HyprPilot.
- Immediate in-flight cancellation and structured progress for longer tasks.
- Acoustic echo cancellation and TTS barge-in.

## Shell surfaces

Delivered in the current tree:

- Always-on bar per monitor, empty-workspace dashboard, and the command panel
  overlay, sharing one design-token module (`shell/theme/Theme.qml`).
- Live workspace list, window counts, focus switching, quick actions, voice-mode
  controls, CPU/memory/GPU/network/disk readouts, and docker/podman state.
- Breakpoint-based responsiveness and a single 1 Hz sampler process.

Still open:

- Launcher search results, task switcher, and notification centre. The bar
  currently opens the command panel; there is no result list behind the search
  field.
- Full per-mode pages behind the bar's mode strip. Only "Terminal" (the
  dashboard) exists; other tabs set `ShellState.mode` without changing content.
- Bring the command panel onto `Theme` and `GlassSurface`. It is still the
  original plain-Qt component because `tests/qml` runs it without Quickshell
  imports, so restyling means giving the test harness an explicit theme fixture.
- Audio/network/power panels through native services.
- Per-surface blur regions (rather than whole-surface compositor blur) and a
  documented reduced-effects mode. `ShellState.reducedEffects` is reserved but
  no module reads it yet.
- A compositor adapter boundary if niri support becomes useful. Hyprland is
  the only implemented compositor backend today.

## Agent work

- Evaluate quantized local planners on real desktop-task fixtures.
- Opt-in accessibility/browser integrations with scoped actions and visible
  approvals; reuse HyprPilot's MCP interface where appropriate.
- Explicitly scoped file tools and an action journal, with review and undo.

Features should earn their persistent CPU/GPU/memory cost. Add benchmarks at
the same time as new always-running work; do not assume Rust or a model's
parameter count proves latency.
