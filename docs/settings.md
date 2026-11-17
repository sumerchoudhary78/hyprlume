# Settings

The **Idle & sleep** page saves opt-in automatic timers and idle brightness.
These preferences apply live; they do not require restarting the daemon. See
[idle/sleep](idle-sleep.md) for presentation mode and external-sleep limits.

The settings button or `hyprlume settings` opens settings using the same glass
components as the desktop. Changes stay in a draft until Save changes; closing
or Reload / discard forgets the draft.

- **Appearance:** accent, scale (0.85–1.20), glass refraction (0–24), reduced
  effects and the system sampler. Saved appearance applies immediately in all
  shell windows. `HYPRLUME_REDUCED_EFFECTS=1` still forces the low-effects path.
- **Voice & models:** English ASR, wake-word and TTS model directories, keyword
  file, inference threads, spoken output, direct-command policy and the optional
  local decision model endpoint/name/timeout. Saving does not download models,
  start the microphone, construct an inference session or restart the daemon.
  Use the existing model tool to install models, then restart explicitly.
- **Shortcuts:** validated modifier/key combinations and selectable Lua or
  hyprlang declarations. Save first, then copy the declarations into your
  compositor configuration and reload it yourself. Hyprlume never edits or
  reloads that configuration, changes live binds, or resolves conflicts with
  your existing binds. Custom socket paths need corresponding CLI arguments.
- **Services:** clipboard-history and Do Not Disturb startup defaults. Current
  states remain controlled by their own panels. Clipboard history is opt-in;
  enabling the startup preference starts collection next time the daemon runs.

Voice/model, launch-allowlist reloads and service startup changes display a
restart notice. Appearance needs no restart. Runtime command plans keep the
immutable configuration from daemon startup: saving settings cannot retarget an
existing AI confirmation or silently change a running model. The microphone
always starts off, including after loading saved preferences.

Use [Input](input.md) for keyboard, pointer and system keys,
[Credentials](credentials.md) for the current keyring provider, and
[Desktop](desktop-preferences.md) for default apps, wallpaper, locale and
accessibility. [Portals](portals.md) shows the current application portal routes.

## Persistence and concurrency

The editor uses the daemon's actual configuration path, including `--config`.
It preserves the configured application argv allowlist; the UI does not edit
executables. Saves validate every value, reject stale revisions and compare the
on-disk bytes with the last loaded version. External edits require Reload.
Files are written and flushed in the same directory, then renamed atomically
with mode 0600. An unsafe directory, symlinked/non-owned file, excessive file
size or I/O error is rejected. This is optimistic conflict detection, not a
transaction shared with unrelated editors. Reformatting removes TOML comments.
A demo daemon changes only its in-memory preview.

The model endpoint remains local HTTP on 127.0.0.1 or ::1. No credentials or
cloud model setup are added here. Missing model directories may be saved in
advance; existing voice prerequisite checks report missing files when used.
Shortcut validation accepts SUPER/CTRL/ALT/SHIFT, letters, digits, named
navigation keys and F1–F24, and rejects duplicate combinations or injected
configuration syntax. Exported Lua uses the current
[Hyprland bind API](https://wiki.hypr.land/configuring/core/binds/).

## Verification

Rust tests cover validation, revisions, permissions, allowlist preservation,
external edit conflicts, reload and restart requirements. The real-socket
`check-settings.py` additionally restarts a private daemon with a custom path
and verifies restored preferences, DND and microphone-off state.
`render-desktop.py --settings --interact` edits production controls with real
input, including decimal scale entry and correlated save acknowledgments.
Full, compact, voice/model and shortcut renders are CI artifacts. The separate
`check-shell.py` starts the actual shell composition root offline in a private
Xvfb/D-Bus environment, catching wiring/import/binding regressions that fixture
screenshots alone cannot establish. None of these tests modifies the installed
session or the user's compositor configuration.
