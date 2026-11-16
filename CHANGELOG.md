# Changelog

## Unreleased

- Add a Hyprland-native desktop shell: a bar on every monitor, a bottom-layer
  dashboard that maps only on an empty workspace, and the existing command panel
  as an overlay. The dashboard covers quick actions, live workspace flow, a
  daemon-fed command stage, the agent panel, system and container cards, a
  workspace rail, and an action ring.
- Ship the whole `shell/` module tree from the AUR package instead of a fixed
  file list.
- Switch workspaces with the dispatcher form Hyprland actually runs: classic
  `.conf` strings or Lua expressions on Hyprland 0.55+, including named and
  special workspaces.
- Add `scripts/check-qml.py` and run it in CI before the Qt test suite.

## 0.1.2 — 2026-11-16

- Fix the graphical-session autostart ordering cycle in the user units.
- Launch apps through separate systemd user units, preserving their lifetime
  across assistant restarts and checking that process execution succeeded.
- Remove unused runtime TLS from the loopback-only model client.

## 0.1.1 — 2026-11-16

- Fix clean Arch package builds by disabling makepkg's GCC LTO for native Cargo
  dependencies. The Rust release profile retains thin LTO.
- Fetch only the build platform's dependencies and explicitly use the stable
  Rust toolchain and package-local target directory.

## 0.1.0 — 2026-11-16

- Initial source preview: Quickshell glass palette/status pill, Rust daemon and
  CLI, typed HyprPilot actions, local English wake/ASR/Piper pipeline, optional
  local model planner, reviewed plans, documentation, and automated tests.
