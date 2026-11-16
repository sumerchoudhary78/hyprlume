# Contributing

Use a current Rust toolchain and keep the checked-in lockfile. Run:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
python3 scripts/check-qml.py shell
```

UI changes must also pass `python3 scripts/qa/render-desktop.py --interact` on
Linux and be rendered with that script at desktop and compact sizes. It runs
Quickshell's real QML widgets on a private Xvfb display; no microphone or real
desktop action is used. See [the desktop guide](docs/homescreen.md) for packages
and reduced-effects checks. Inspect exported PNGs before submitting a visual
change. GitHub CI uploads the same images as artifacts.

Keep shared visuals in `shell/components`, composition in `shell/modules`,
processes/IPC in `shell/services`, and all asset origins in `shell/assets/README.md`.
Do not bake interactive UI into image assets. The explicit preview fixture
must never be imported by the live shell. Runtime packaging excludes fixtures.

New desktop actions must use typed, bounded inputs, existing preview/confirmation
rules and observable postconditions. Never pass model text to a shell. Add tests
for rejected input and failed postconditions, not just successful dispatch.
