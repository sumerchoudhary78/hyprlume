# Security

Report sensitive issues privately through GitHub's vulnerability reporting for
this repository when available. Do not post private recordings or credentials.

The daemon runs as the logged-in user. Its private socket, typed action list,
launch allowlist, preview tokens, and resource limits reduce accidental or
model-generated misuse; they are not an OS sandbox against same-user malware.
Wake words do not authenticate a speaker.

Microphone capture requires explicit activation each run. Audio stays local.
Dictation never dispatches desktop commands. The model adapter is loopback-only
and model plans require confirmation. No root command, shell execution tool,
arbitrary keyboard injection, file deletion, or outbound-message tool is
exposed by this release.

Trust configured launchers and the separately running model server. Review
all action previews, especially in shared/noisy environments. See
[architecture](docs/architecture.md) and [voice behavior](docs/voice.md) for
current limits.
