# Sound and media

Available in the v0.5.0 desktop preview.
Open **Sound & media** from the speaker in the bar, or run `hyprlume audio`
from a matching development build. Opening the panel does not alter audio.

## Supported controls

- Enumerate PipeWire audio outputs, inputs and application streams.
- Set each bound node's volume between 0 and 100%, or toggle its mute state.
  Existing externally amplified values remain visible; the shell never increases
  them above 100%. Left/right keys adjust a focused slider by 1%.
- Select a preferred default output or input. WirePlumber applies that preference
  according to its routing policy; existing streams are not forcibly rewired.
- Select among MPRIS players; display plain-text title, artist and album; send
  previous, play/pause and next only when that player advertises support.
- Keep selection on the chosen player. When it disappears, fall back to a
  playing player, then the first remaining player.

The output vessel visualizes volume, not microphone activity or measured audio
amplitude. There is no peak capture or extra recording stream. Player artwork
URLs are not fetched, and metadata is never interpreted as markup or commands.

## Ownership and cost

`shell/services/NativeAudio.qml` reuses Quickshell's event-driven PipeWire and
MPRIS adapters. `AudioController.qml` owns selection and control validation;
`shell/modules/audio/` owns presentation. Rust owns the mutually exclusive
panel state and continues to execute checked agent volume/mute commands through
the existing desktop adapter. No custom C++ or second PipeWire client library
was added to Hyprlume.

Controls keep native object references. A removed, unbound or unavailable node
is rejected before mutation; a player must still be present and capable of the
requested action. Numeric PipeWire IDs are not passed through subprocesses.
The closed panel binds only the current output and input. Opening it binds at
most 64 audio nodes; the player view is capped at 24. There are no shell-command
volume polls, progress timers, automatic player switching while a selection
exists, or continuously animated sound meters.

Panel opening/closing preserves an agent's pending confirmation. Command,
wireless, notifications and sound overlays exclude each other. Losing the
daemon connection hides the overlay. MPRIS discovery belongs to Quickshell and
does not require playback to be active or PipeWire to be available.

## Current limits

This milestone does not expose per-stream routing, device profile/port editing,
channel balance, amplification above 100%, track seeking, playlists or saved
player preferences. Native Quickshell property setters expose observed state
but no per-request completion/error receipt; asynchronous server errors remain
in the shell log. The panel does not claim a change was confirmed in a toast.

Physical HDMI/Bluetooth/USB handoff, WirePlumber policy differences, device
hot-plug/reconnection, all media players and long-running CPU/GPU/frame pacing
still need qualification. The active Arch desktop was not changed for testing.

## Reproduce isolated validation

On Arch, the test tools include `python-dbus-next`, `pipewire`, `quickshell`,
`xorg-server-xvfb`, `xorg-xauth`, `dbus` and `xdotool`, plus the project's existing
Qt Quick 3D/shader dependencies. Run from a source checkout:

```sh
python3 scripts/qa/check-audio.py
python3 scripts/qa/render-desktop.py --audio --interact
python3 scripts/qa/render-desktop.py --audio --output output/design/audio.png
python3 scripts/qa/render-desktop.py --audio --width 960 --height 640 --reduced-effects --output output/design/audio-compact.png
python3 scripts/qa/render-desktop.py --audio --audio-state unavailable --output output/design/audio-unavailable.png
```

The native adapter test creates a temporary runtime directory, private D-Bus,
private Xvfb display and software-only PipeWire server. Its config loads no
ALSA/Bluetooth/portal or hardware-discovery module. A fake MPRIS app receives
real D-Bus playback calls. Independent `pw-dump` and `pw-metadata` queries check
server volume, mute and the configured default. Child processes are terminated
on success, failure or timeout. The shared-controller checks reject invalid
volume, missing/unbound devices, stale players and unsupported actions.

Pointer/keyboard tests use `AudioFixture.qml`, exercising the production QML
controls without contacting real devices, players, the microphone or speaker.
