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

- A per-output status bar, interactive window maps on empty workspaces,
  and an atlas you can open over applications. [UI design record](design/shell-ui.md)
- Workspace switching, inline command previews, voice controls and glanceable
  CPU, memory and GPU readings. System details expose network, disk and
  Docker/Podman status. The system sampler pauses when the expanded atlas closes.
- Shared Qt Quick controls, keyboard focus indicators, reduced effects, high
  contrast and isolated native input tests.
- Physical-session checks for wallpaper/bar layers, workspace switching, focus,
  window moves and dashboard hide/restore on Hyprland 0.57.1 Lua.
- The first desktop-service panel: Wi-Fi/Bluetooth through Rust NetworkManager
  and BlueZ adapters, with scans, connection controls and explicit pairing
  prompts. See [supported modes and qualification](connectivity.md).

Still open:

- Persistent, task-specific workflows beyond the existing command palette.
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
2. **Notifications:** available in v0.5.0: D-Bus ownership, actions,
   dismissal, bounded session history and Do Not Disturb. Markup, images, sounds,
   durable settings/history and activation tokens remain outside this milestone.
   See [notification behavior and isolation](notifications.md).
3. **Audio/media:** available in v0.5.0: PipeWire devices, preferred
   input/output, node volume/mute and capability-aware MPRIS playback. Per-stream
   routing, profiles/ports and physical hot-plug qualification remain. See
   [supported controls and isolated checks](audio.md).
4. **Power/session:** available in v0.5.0: battery/UPS, kernel
   brightness, power profiles and separately confirmed suspend/logout/restart/
   shutdown. Physical driver, policy, sleep/resume and login teardown
   qualification remain. Suspend now waits for secure locking. See
   [power behavior and isolation](power.md).
5. **System tray:** available in v0.5.0: StatusNotifier icons,
   activation/secondary/scroll actions and themed DBusMenu navigation with
   check/radio states. Legacy XEmbed and the physical app matrix remain outside
   this milestone. See [tray behavior and isolated checks](tray.md).
6. **App launcher:** available in v0.5.0: native desktop-entry indexing,
   ranked application search and exact-address window switching. See
   [behavior and isolated checks](launcher.md).
7. **Clipboard/capture:** available in v0.5.0: opt-in bounded text
   history, screenshots, display/region recording and persistent stop controls.
   See [behavior, dependencies and isolated checks](clipboard-capture.md).
8. **Settings:** available in v0.5.0: persisted appearance, voice/
   model preferences, shortcut exports and service startup defaults. See
   [application semantics and verification](settings.md).
9. **Screen locking:** available in v0.5.0: dedicated ext-session-lock/
   PAM supervision, themed authentication, action/voice/capture pausing and
   fail-closed recovery. See [security behavior and qualification](locking.md).
10. **Idle/sleep:** available in v0.5.0: opt-in dim/lock/screen-off/
    suspend timers, live settings, presentation and recording inhibition,
    conditional restoration and confirmed lock-before-suspend. External sleep
    locking has a bounded logind delay; physical lid/sleep/driver qualification
    remains. See [idle policy and guarantees](idle-sleep.md).

11. **Polkit:** available in v0.5.0. The optional Wayland dialog
    uses the system authentication helper. See [setup and verification](polkit.md).

12. **Desktop portals:** available in v0.5.0. The opt-in
    profile routes to Hyprland and GTK and publishes Hyprlume appearance hints.
    See [setup, live diagnostics and verification](portals.md).

13. **Input and system keys:** persisted keyboard and pointer options, a
    managed Hyprland include and glass feedback for volume, brightness and media.
    See [input and system keys](input.md).

14. **Credential storage:** Secret Service discovery, login and default
    collection controls and provider-owned prompts. See [credential storage](credentials.md).

15. **Removable storage:** event-driven UDisks2 controls for mounting,
    encryption and safe removal. See [removable drives](storage.md).

16. **Desktop preferences:** GIO app defaults, file associations, wallpaper,
    app appearance, locale, clock and accessibility. See [desktop preferences](desktop-preferences.md).

17. **Selectable session:** UWSM startup and shutdown, session environment, XDG
    autostart and gated Hyprlume services. See [session setup](session.md).

18. **Display settings:** resolution, refresh, scale and arrangement with a
    verified preview and timed rollback. Managed logins restore remembered
    connected-output layouts. See [display settings](displays.md).

Stable-release qualification covers physical devices, docking, suspend and graphics performance. See [validation](validation.md).

## Agent work

- Evaluate quantized local planners on real desktop-task fixtures.
- Opt-in accessibility/browser integrations with scoped actions and visible
  approvals; reuse HyprPilot's MCP interface where appropriate.
- Explicitly scoped file tools and an action journal, with review and undo.

Features should earn their persistent CPU/GPU/memory cost. Add benchmarks at
the same time as new always-running work; do not assume Rust or a model's
parameter count proves latency.
