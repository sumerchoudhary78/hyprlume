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
- The first desktop-service panel: Wi-Fi/Bluetooth through Rust NetworkManager
  and BlueZ adapters, with scans, connection controls and explicit pairing
  prompts. See [supported modes and qualification](connectivity.md).

Still open:

- Full task-specific pages and persisted workflows behind navigation. Existing
  tabs select the console, action activity, assistant focus or live system data.
- Remaining desktop services in the implementation sequence below.
- A compositor adapter boundary if niri support becomes useful. Hyprland is
  the only implemented compositor backend today.

## Desktop-service sequence

Services use Rust controllers where needed or existing event-driven Quickshell
adapters. Each has a separate interface, themed controls, explicit unavailable
states and native UI tests before the next one starts. No custom C++ service
code is needed. User-selected priority is connectivity:

1. **Wi-Fi/Bluetooth:** implemented; physical connection/pairing qualification,
   hidden/enterprise profile editing and Bluetooth battery indicators remain.
2. **Notifications:** implemented on main (unreleased): D-Bus ownership, actions,
   dismissal, bounded session history and Do Not Disturb. Markup, images, sounds,
   durable settings/history and activation tokens remain outside this milestone.
   See [notification behavior and isolation](notifications.md).
3. **Audio/media:** implemented on main (unreleased): PipeWire devices, preferred
   input/output, node volume/mute and capability-aware MPRIS playback. Per-stream
   routing, profiles/ports and physical hot-plug qualification remain. See
   [supported controls and isolated checks](audio.md).
4. **Power/session:** implemented on main (unreleased): battery/UPS, kernel
   brightness, power profiles and separately confirmed suspend/logout/restart/
   shutdown. Physical driver, policy, sleep/resume and login teardown
   qualification remain. Secure locking is a separate milestone. See
   [power behavior and isolation](power.md).
5. **System tray:** implemented on main (unreleased): StatusNotifier icons,
   activation/secondary/scroll actions and themed DBusMenu navigation with
   check/radio states. Legacy XEmbed and the physical app matrix remain outside
   this milestone. See [tray behavior and isolated checks](tray.md).
6. **App launcher:** implemented on main (unreleased): native desktop-entry indexing,
   ranked application search and exact-address window switching. See
   [behavior and isolated checks](launcher.md).
7. **Clipboard/capture:** implemented on main (unreleased): opt-in bounded text
   history, screenshots, display/region recording and persistent stop controls.
   See [behavior, dependencies and isolated checks](clipboard-capture.md).
8. **Settings:** implemented on main (unreleased): persisted appearance, voice/
   model preferences, shortcut exports and service startup defaults. See
   [application semantics and verification](settings.md).
9. **Screen locking:** a real session-lock integration with authentication and
   recovery qualification; a fullscreen panel alone cannot provide a lock.

## Agent work

- Evaluate quantized local planners on real desktop-task fixtures.
- Opt-in accessibility/browser integrations with scoped actions and visible
  approvals; reuse HyprPilot's MCP interface where appropriate.
- Explicitly scoped file tools and an action journal, with review and undo.

Features should earn their persistent CPU/GPU/memory cost. Add benchmarks at
the same time as new always-running work; do not assume Rust or a model's
parameter count proves latency.
