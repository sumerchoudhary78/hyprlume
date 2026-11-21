# Desktop shell

An empty workspace shows the workspace atlas on its wallpaper. The maps draw the positions and sizes of the actual windows on each workspace. Click a window shape to focus it or click its workspace name to switch there. An occupied workspace has the wallpaper and a 40-logical-pixel bar at the default scale.

## Everyday controls

| Control | What it does |
| --- | --- |
| Workspace name or Hyprlume mark at the top left | Opens **Workspace atlas** over any app. Use the arrow keys to select a map, Home or End to reach the first or last workspace, and Enter to switch. **Window list** gives each window a keyboard-accessible action by title. |
| Apps | Opens search with the cursor ready. Type, use the arrow keys to select a result, and press Enter. The Windows tab switches to an open window. |
| Command | Opens the command palette. Enter previews an action. **Run plan** confirms it. The explicit **Run direct** action is available for direct commands. |
| Clock | Opens notifications. A bell indicates unread items and a moon indicates Do Not Disturb. |
| Status icons at the top right | Open **Quick settings**. Adjust or mute the default output, open Connections or Sound, change microphone or Do Not Disturb state, and reach Desktop tools, App tray, Settings, Lock or Power. |
| Active microphone or recording indicator | Turns off the microphone or stops screen recording directly from the bar. |

The expanded atlas also has a command field, a shortcut to preview the configured terminal alias, and local dictation. Its System section displays CPU, memory and GPU readings. Expand it for network, disk, load, uptime, containers and CPU history. The sampler runs only while the expanded atlas is open with metrics enabled.

In the starter session, **Super+W** toggles the atlas and **Super+,** toggles Quick Settings. Escape or a click outside dismisses either. A small layout or one with missing geometry can still be reached by title in the window list. Custom sessions can call the same IPC handlers.

```sh
quickshell ipc -p /usr/share/hyprlume/shell/shell.qml call desktop overview
quickshell ipc -p /usr/share/hyprlume/shell/shell.qml call desktop quickSettings
```

## Display and appearance

Narrow outputs keep one row by dropping optional labels and status details. Menus fit the available height and scroll with keyboard focus. Shell scale adjusts control density independently of the compositor's monitor scale. [Qt high DPI](https://doc.qt.io/qt-6/highdpi.html)

**Settings → Desktop** lets you choose a local wallpaper or restore Caldera. The bundled JPEG is 1672×941. Qt renders the live maps, text and controls at the output's device pixel ratio. Reduced effects and high contrast turn off menu entrance motion and wallpaper sampling. [Design decisions](design/shell-ui.md)

| Namespace | Layer | Lifetime |
| --- | --- | --- |
| `hyprlume-wallpaper` | Background | One per output. Only the map controls on an empty workspace receive pointer input. The surface never takes keyboard focus. |
| `hyprlume-bar` | Top | One per output, reserving its rendered height |
| `hyprlume-desktop-menu` | Overlay | While the expanded atlas or Quick Settings is open |
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
