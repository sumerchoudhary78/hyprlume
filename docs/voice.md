# Local English voice

## Modes

| Mode | Microphone | Recognition | Desktop actions |
| --- | --- | --- | --- |
| Off (startup default) | No capture process | None | Typed commands only |
| Wake | Continuous capture | Small keyword model until “Hey Pilot”, then streaming ASR | Preview by default |
| Dictate | Continuous capture | Streaming ASR | Never; selectable transcript only |

Use the pill/panel controls or `hyprlume voice off` to stop capture. Enabling
voice is an explicit user action and is not persisted across daemon restarts.
Dictation currently displays the latest utterance, rather than saving a full
meeting transcript or automatically typing into another application.

In wake mode, say **“Hey Pilot”**, wait for the listening indicator, then speak a
command such as “switch to workspace two”. Each command ends after a short
silence, with a hard time limit. A new command requires another wake phrase.

## Models

`hyprlume-models` downloads three upstream bundles (~395 MB total compressed),
checks their pinned SHA-256 digests, and extracts them with a safe data filter.
It does not execute downloaded scripts, start audio capture, or overwrite an
existing custom model directory. It keeps the upstream model cards.

| Purpose | Model | Runtime |
| --- | --- | --- |
| Wake | Zipformer GigaSpeech 3.3M, int8 | sherpa-onnx CPU, one thread |
| English streaming ASR | Zipformer English 2023-06-26, int8 | sherpa-onnx CPU, two threads by default |
| Speech output | Piper `en_US-lessac-medium` | sherpa-onnx CPU; external eSpeak IPA phonemizer |

Model files live under `$XDG_DATA_HOME/hyprlume/models` (normally
`~/.local/share/hyprlume/models`). The native inference library is part of the
build; weights are not part of the package. Consult upstream licenses/model
cards before redistributing weights or replacing voices.

The default wake phrase is encoded in `kws/hyprlume-keywords.txt`; the original
upstream keywords file is preserved. Changing the phrase requires tokenization
using that model's `bpe.model`, as explained in
[sherpa-onnx keyword spotting](https://k2-fsa.github.io/sherpa/onnx/kws/index.html).
Plain words in that file are not sufficient.

Use `--cache PATH --offline` with `hyprlume-models` to install previously
downloaded, checksummed archives without network access. `--only asr`, `kws`, or
`tts` installs one group. All paths can be overridden in `[voice]`; use absolute
paths, as TOML strings do not expand `~` or shell variables.

## Applying voice commands

The default is review before execution. To let recognized **direct commands**
apply automatically, explicitly set:

```toml
[voice]
execute_commands = true
```

Restart the daemon, then re-enable wake mode. Model-generated plans still
require confirmation. Commands cannot run shell text, delete files, send
messages, or synthesize arbitrary keystrokes in this release.

Wake detection is not speaker authentication. TV/audio or another person can
trigger it. Keep preview mode if that is a concern in your environment.

## Audio behavior and current limits

PipeWire supplies 16 kHz mono PCM in 20 ms frames. A bounded queue feeds a
dedicated inference worker. Raw microphone audio is neither written to disk
nor sent over the network. Only recognized command text reaches the planner.
Transcripts remain in bounded in-memory UI state until replaced or the daemon
exits. Debug logs do not record microphone audio.

Spoken feedback temporarily suspends recognition and drains its short echo
backlog before re-arming the keyword spotter. **True acoustic echo cancellation
and interrupting TTS by speaking are not implemented.** During feedback the
audio stream stays open, but recognition discards its frames. Use the visible
mic-off control to end capture completely.

TTS produces a temporary WAV in the private runtime directory for `pw-play` and
removes it after playback. Explicit `hyprlume speak --output PATH` keeps the
requested file. TTS models are cached after first use inside the daemon.

Recognition accuracy depends on accent, microphone, and noise. The small
English model is a starting point for testing, not an accuracy guarantee.
False-wake rate and physical microphone latency need real-session evaluation.
