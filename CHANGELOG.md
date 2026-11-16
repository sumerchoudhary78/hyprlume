# Changelog

## 0.1.1 — 2026-11-16

- Fix clean Arch package builds by disabling makepkg's GCC LTO for native Cargo
  dependencies. The Rust release profile retains thin LTO.
- Fetch only the build platform's dependencies and explicitly use the stable
  Rust toolchain and package-local target directory.

## 0.1.0 — 2026-11-16

- Initial source preview: Quickshell glass palette/status pill, Rust daemon and
  CLI, typed HyprPilot actions, local English wake/ASR/Piper pipeline, optional
  local model planner, reviewed plans, documentation, and automated tests.
