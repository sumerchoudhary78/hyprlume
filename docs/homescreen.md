# Desktop shell

Every output shows the wallpaper and a single status bar. At the default shell scale the bar reserves 40 logical pixels. The central workspace stays clear until you open an app or a menu.

## Everyday controls

| Control | What it does |
| --- | --- |
| Workspace name or Hyprlume mark at the top left | Opens **Overview**. Choose a workspace to switch to it. Each row gives its window count. |
| Apps | Opens search with the cursor ready. Type, use the arrow keys to select a result, and press Enter. The Windows tab switches to an open window. |
| Command | Opens the command palette. Enter previews an action. **Run plan** confirms it. The explicit **Run direct** action is available for direct commands. |
| Clock | Opens notifications. A bell indicates unread items and a moon indicates Do Not Disturb. |
| Status icons at the top right | Open **Quick settings**. Adjust or mute the default output, open Connections or Sound, change microphone or Do Not Disturb state, and reach Desktop tools, App tray, Settings, Lock or Power. |
| Active microphone or recording indicator | Turns off the microphone or stops screen recording directly from the bar. |

Overview also has a command field, a shortcut to preview the configured terminal alias, and local dictation. Its System section displays CPU, memory and GPU readings. Expand it for network, disk, load, uptime, containers and CPU history. The sampler runs only while Overview is open with metrics enabled.

In the packaged starter session, **Super+W** toggles Overview and **Super+,** toggles Quick Settings. Escape or a click outside dismisses either menu. The menus also work over open application windows. Custom session configurations can call the same handlers with their own bindings.

```sh
quickshell ipc -p /usr/share/hyprlume/shell/shell.qml call desktop overview
quickshell ipc -p /usr/share/hyprlume/shell/shell.qml call desktop quickSettings
```

## Display and appearance

Narrow outputs keep one row by dropping optional labels and status details. Menus fit the available height and scroll with keyboard focus. Shell scale adjusts control density independently of the compositor's monitor scale. [Qt high DPI](https://doc.qt.io/qt-6/highdpi.html)

**Settings → Desktop** lets you choose a local wallpaper or restore First Light. The bundled JPEG is 1672×941. Qt renders text and controls at the output's device pixel ratio. Reduced effects and high contrast turn off menu entrance motion and wallpaper sampling. [Design decisions](design/shell-ui.md)

| Namespace | Layer | Lifetime |
| --- | --- | --- |
| `hyprlume-wallpaper` | Background | One per output, with an empty input region |
| `hyprlume-bar` | Top | One per output, reserving its rendered height |
| `hyprlume-desktop-menu` | Overlay | While Overview or Quick Settings is open |
| `hyprlume-command` | Overlay | While the command panel is open |

## Reproduce

On Arch the renderer uses Quickshell, Xvfb, Xauth, D-Bus, xdotool, Pillow, Inter and JetBrains Mono. It runs in a private display and bus with synthetic data.

```sh
python3 scripts/check-qml.py shell
python3 scripts/qa/check-shell.py
python3 scripts/qa/render-desktop.py --width 1920 --height 1080 --dpr 2 --output output/design/desktop-4k.png
python3 scripts/qa/render-desktop.py --menu overview --output output/design/overview.png
python3 scripts/qa/render-desktop.py --menu controls --output output/design/quick-settings.png
python3 scripts/qa/render-desktop.py --interact
python3 scripts/qa/render-desktop.py --interact --width 700 --height 620 --dpr 1.5 --reduced-effects
python3 scripts/qa/render-desktop.py --interact --width 520 --height 640 --scale 1.20 --high-contrast
```
