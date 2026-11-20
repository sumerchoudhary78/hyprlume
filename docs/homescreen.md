# Desktop shell

Hyprlume uses a status bar on every output and a command dashboard on the
focused empty workspace. The dashboard opens with the workspace name, a
command field, shortcuts, workspace switching and system readings. See the
[UI design record](design/shell-ui.md) for the research and framework decision.

## Everyday controls

- Select **Apps** to search installed applications and open windows. Select
  **Command** for the full palette, or enter a request directly on Home.
- Enter previews an action. The command panel displays the plan and requires
  **Run plan** before it applies. **Run direct** is an explicit action in the
  full palette. Model-generated plans always require review.
- **Open terminal** previews the configured terminal alias. Configure
  `[apps].terminal` in `~/.config/hyprlume/config.toml`, for example
  `terminal = ["foot"]`.
- **Dictate** starts local dictation. The status bar distinguishes microphone
  off from active capture. The command panel provides the wake-word,
  dictation and microphone-off controls.
- Select a workspace to focus it. Its window count comes from Hyprland.
  **System** shows CPU, memory and GPU. Its disclosure button opens network,
  disk, load, uptime and local container information. Turn off system metrics
  in Settings to hide the card and stop sampling.
- The bar also opens Connections, sound and media, notifications, the tray,
  clipboard and capture, Settings, and power controls. The recording stop
  control remains available in the bar after the capture panel closes.

## Layout and rendering

The wide layout pairs the command card with one sidebar. Narrow displays stack
the same controls, and short displays scroll. Keyboard focus brings clipped
controls into view. The bar reserves the actual space occupied by its rows.
Typography and interactive targets follow the shell’s scale preference in
logical pixels.

For a 3840×2160 display, start at 200% monitor scale for a 1920×1080 logical
workspace. Use the separate shell scale control to adjust the density of
Hyprlume’s controls. Qt renders the text, vector icons and system chart at
the window’s device pixel ratio. [Qt high DPI](https://doc.qt.io/qt-6/highdpi.html) ·
[Hyprland display scaling](https://wiki.hypr.land/configuring/core/monitors/positioning/)

Home uses the Tidal Eclipse wallpaper and may sample it at half resolution
behind the command card. Live Wayland draws the wallpaper on its background
layer and loads a smaller, hidden copy only when the glass needs it. A custom
4K wallpaper retains its source detail on a 4K output. The bundled image is
1672×941. [Asset origins](../shell/assets/README.md)

Panels open with a short position transition while their controls remain
available. High contrast and reduced effects skip the transition and glass
refraction. Set `HYPRLUME_REDUCED_EFFECTS=1` to start in reduced-effects mode.
The power panel also uses this preference for the battery gauge.

| Namespace | Layer | Lifetime |
| --- | --- | --- |
| `hyprlume-wallpaper` | Background | One per output with an empty input region |
| `hyprlume-bar` | Top | One per output, reserving its rendered height |
| `hyprlume-home` | Bottom | Focused output on an empty workspace |
| `hyprlume-command` | Overlay | While the command panel is open |

The dashboard unmaps and metrics sampling pauses when a window opens. The
wallpaper and bar persist. The ordinary-window preview and native design
fixture render the same controls as the Wayland shell. The fixture labels
its data and uses synthetic state.

## Reproduce

On Arch, install `quickshell`, `xorg-server-xvfb`, `xorg-xauth`,
`dbus`, `xdotool`, `python-pillow`, `inter-font` and
`ttf-jetbrains-mono`.

```sh
python3 scripts/check-qml.py shell
python3 scripts/build-shaders.py --check
python3 scripts/qa/check-effects.py
python3 scripts/qa/render-desktop.py --width 1920 --height 1080
python3 scripts/qa/render-desktop.py --width 1920 --height 1080 --dpr 2 --output output/design/desktop-4k.png
python3 scripts/qa/render-desktop.py --command-review --output output/design/command-review.png
python3 scripts/qa/render-desktop.py --interact
python3 scripts/qa/render-desktop.py --interact --width 700 --height 620 --dpr 1.5 --reduced-effects
python3 scripts/qa/render-desktop.py --width 520 --height 640 --scale 1.20 --high-contrast --output output/design/high-contrast.png
```

The commands use a private Xvfb display, runtime directory and D-Bus session.
GitHub CI exercises the same QML and uploads renders. Physical compositor and
performance qualification is tracked in [validation](validation.md).
