# Architecture

## Processes and ownership

Quickshell owns rendering, input widgets, and layer-shell surfaces. The Rust
daemon owns configuration, the bounded action planner/executor, model work,
and observed desktop state. A CLI sends the same protocol as the UI.

The components communicate over a private Unix socket at
`$XDG_RUNTIME_DIR/hyprlume/control.sock`. Its directory is mode 0700 and the
socket is mode 0600. A file lock prevents a second daemon replacing a live
instance. Only a socket, never a regular file or symlink, can be replaced on
restart. At most 16 clients connect, with bounded frames and write deadlines.

The UI subscribes once. A Tokio watch channel coalesces state into the latest
snapshot rather than retaining an unbounded event queue. Reconnect timers run
only while disconnected. The daemon reads Hyprland's event socket through
HyprPilot, without a polling loop for desktop status.

## HyprPilot integration

`hyprpilot-core` is an actual Cargo dependency pinned to commit
`de1f0f4ae25aabe3fcf4964150eb33c1386cd9d4` from
[HyprPilot](https://github.com/sumerchoudhary78/HyprPilot).
Queries, selectors, workspace dispatchers, focus, window moves, and compositor
events use that implementation. Hyprlume does not copy a second Hyprland IPC
client into this repository.

HyprPilot's daemon, persistent undo, snapshots, accessibility, and MCP server
remain useful adjacent integrations. This release does not start or expose
those broader capabilities. They are not required to run Hyprlume, and the
installed `hyprpilot-mcp` package is not a runtime dependency of this shell.

## Planning and execution

1. Validate plain text and attempt direct command parsing.
2. For an unmatched request, use the explicitly configured local model server.
3. Deserialize and validate a plan of at most four known actions. Launch
   names must resolve to trusted configured argv arrays.
4. Preview, with a single-use confirmation token. Model plans always preview.
5. Apply actions serially. Stop the batch on the first error and retain any
   receipts for earlier completed actions.
6. Query the desktop to check observable workspace/window/audio state.

The launch receipt confirms process creation, not that a graphical window has
finished loading. A move binds its target before execution and a move preview
checks for a focus change before confirmation. Later actions keep that target
even if a preceding workspace action changes focus.
Batch execution is not transactional, and previous effects are not rolled back
when a later action fails. Preview dismissal is available before execution;
in-flight cancellation and undo are roadmap work.

## Voice

A dedicated blocking worker owns the native speech objects. The async reader
pushes 20 ms PCM frames into an eight-frame queue, with backpressure. Only the
small keyword model decodes while waiting for a wake phrase. ASR decodes after
activation; dictation bypasses the keyword gate and never reaches the executor.

Native model inference does not run on the UI thread. Piper speech output is
cached; a separate eSpeak process performs phonemization. Capture is suspended
from recognition during spoken feedback, avoiding immediate self-triggering.
See [voice](voice.md) for precise limitations.

## Trust boundary

The protocol constrains what Hyprlume can do on behalf of a transcript/model.
It is not a sandbox against arbitrary programs running as the same user.
The daemon and its allowed launchers retain that user's normal desktop rights.
There is no authentication claim based on the wake word.

The local LLM adapter sends no screenshots or arbitrary file data, accepts no
redirects or non-loopback endpoints, and bounds its response. A loopback model
server must still be treated as a separately administered local program.
