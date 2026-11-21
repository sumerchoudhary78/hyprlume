# Applications and window switching

The Apps button opens the launcher with keyboard focus in search. Applications are
indexed by Quickshell's native XDG desktop-entry service; Hidden and NoDisplay
entries are excluded. Search matches names, generic names, comments, IDs and
keywords, with title prefixes ranked first. It returns at most 32 results.
There is no background per-keystroke process, usage tracking or network query.

The Windows tab searches currently open Hyprland windows. Up/Down chooses a
result, Enter activates it, and Escape closes the surface. Clicks work too.
`hyprlume launcher` opens applications; `hyprlume launcher --windows` opens
windows. For example, add bindings yourself to your Hyprland configuration:

```ini
bind = SUPER, SPACE, exec, hyprlume launcher
bind = SUPER, TAB, exec, hyprlume launcher --windows
```

These examples use hyprlang syntax; Lua configurations need the corresponding
exec bindings. Hyprlume never edits or reloads the compositor configuration.

App activation sends a validated desktop ID to Rust. `gtk-launch` (Arch package
`gtk3`) delegates field codes, desktop activation and terminal launching to GIO.
The launcher runs in a separate systemd user unit with `ExitType=cgroup`, so a
forking app is owned independently of the shell and daemon. A successful
request means that the launcher process started; it does not prove an app
created a window. Broken desktop files, absent terminal handlers or app startup
failures still need the app's own logs. Entries using uncommon ID characters
outside letters, digits, dot, underscore and hyphen are rejected explicitly.

Window switching binds the exact hexadecimal address, checks that it still
exists, dispatches through the HyprPilot transport (Lua or hyprlang), and reads
the active window back. An acknowledgment alone is not success. The manual app
launcher does not extend the AI command allowlist or authorize model-generated
executables. Desktop files have the same trust as apps installed in your XDG
application directories.

## Verification

`scripts/qa/check-launcher.py` creates private XDG application directories,
Xvfb and D-Bus. It tests index visibility, keyword search and stale selections,
then launches a harmless temporary marker fixture through real GIO and verifies
Exec field-code expansion. `render-desktop.py --launcher --interact` exercises
the production QML with synthetic apps/windows and real keyboard/mouse input.
Rust fake-compositor tests verify exact-address dispatch and postconditions.
These checks never launch installed user applications or change desktop focus.
Physical Wayland focus behavior, terminal handlers and the distribution's app
matrix remain release-qualification work.
