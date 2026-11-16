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

- Launcher search, task switcher, dock, workspace overview, and notifications.
- Consistent glass design tokens with accessibility and a reduced-effects mode.
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
