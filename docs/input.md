# Input and system keys

Open **Settings → Input** and save your keyboard and pointer preferences. Use XKB identifiers such as `us,gb` and optional positional variants such as `intl,`. Hyprlume validates them against the installed `xkeyboard-config` catalog. The keyboard list shows the active keymap reported by Hyprland. **Next layout** and `hyprlume input --next-layout` cycle the currently configured layouts.

**Apply input now** sends the saved keyboard, pointer and touchpad options to the running Hyprland instance. Per-device Hyprland settings can take precedence. A compositor reload can reset live options.

## Persistent configuration and shortcuts

1. Enable **Include my Hyprlume shortcuts** and **Include system media, volume and brightness keys** if you want the profile to manage those bindings. Both are opt-in.
2. Save, then choose **Write profile**. Hyprlume checks the visible compositor bindings for conflicting named chords and shows the generated path.
3. Include the profile once, after your input settings. In Lua, use `dofile("/absolute/path/to/hyprlume-input.lua")`. In hyprlang, use `source = /absolute/path/to/hyprlume-input.conf`. Remove duplicate declarations that you previously installed for these same bindings, then reload Hyprland. Subsequent profile writes can trigger Hyprland’s auto-reload depending on your settings.

Keep the optional copied declarations in **Settings → Shortcuts** if you manage bindings manually. Verify custom keycodes and conditional bindings in your Hyprland config before enabling the profile.

Volume and microphone keys use WirePlumber. Media keys use optional `playerctl` for MPRIS applications. Brightness keys adjust the first kernel backlight from the verified graphical session. Each action shows short, click-through glass feedback. Key actions are paused during session locking.
