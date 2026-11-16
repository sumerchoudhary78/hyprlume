# Contributing

Keep changes small enough to measure and review. Open an issue before adding
a new persistent service, renderer, model family, or compositor backend.

Run formatting, clippy with warnings denied, and the Rust test suite. UI
changes should also pass `qmltestrunner -input tests/qml` under Qt 6, including
the offscreen backend for CI. Capture the actual native panel after layout
changes using `shell/preview.qml` with `HYPRLUME_SCREENSHOT` set to a PNG path.

Use typed actions with explicit bounds. Do not turn model/transcript strings
into shell commands. Add a user-facing validation/receipt path for desktop
mutations, and test unknown input and failed dependencies.

Do not add periodic polling where the service exposes events. Keep model work
off the UI thread, queues bounded, and effects inactive while surfaces are
hidden. Report the measurement environment and separate measured results from
targets. Never commit model weights, transcripts, private configuration, API
keys, or recordings of other people.
