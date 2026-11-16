# Roadmap

The first release establishes a usable command/voice path. A full desktop shell
and an unrestricted computer-use agent are larger projects.

## Next: qualify the interactive path

- Physical-microphone tests for false wakes, accents, noisy rooms, silence,
  device disconnects, and sleep/resume.
- Longer Hyprland/NVIDIA runs, fractional scaling, multiple monitors and
  compositor frame-time measurements beyond the verified single-monitor path.
- Richer target selection and persistent undo/snapshot integration with HyprPilot.
- Immediate in-flight cancellation and structured progress for longer tasks.
- Acoustic echo cancellation and TTS barge-in.

## Shell surfaces

Delivered in the current tree:

- Always-on bar per monitor, empty-workspace dashboard, and the command panel
  overlay, sharing one design-token module (`shell/theme/Theme.qml`).
- Live workspace list, window counts, focus switching, quick actions, voice-mode
  controls, CPU/memory/GPU/network/disk readouts, and docker/podman state.
- A compact layout, reduced-effects mode and one two-second sampler that pauses
  while the dashboard is hidden.
- Cached wallpaper glass, procedural 3D globe/city and the same command component
  used by the live shell and native input tests.
- Physical-session checks for wallpaper/bar layers, workspace switching, focus,
  window moves and dashboard hide/restore on Hyprland 0.57.1 Lua.

Still open:

- Launcher search results, task switcher, and notification centre. The bar
  currently opens the command panel; there is no result list behind the search
  field.
- Full task-specific pages and persisted workflows behind navigation. Existing
  tabs select the console, action activity, assistant focus or live system data.
- Audio/network/power panels through native services.
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
