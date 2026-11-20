# Hyprlume shell UI design

Hyprlume’s empty workspace should make it easy to start a desktop command,
switch workspaces, or check the system at a glance. The redesigned shell uses
an aligned command surface, a compact status bar and a dark coastal wallpaper.
The workspace name and command field lead the page. System readings and
desktop services sit one level below, with details available on demand.

## Research and decisions

| Evidence | Decision for Hyprlume |
| --- | --- |
| The v0.5.1 desktop and Connections renders stack a detailed observatory, tilted cards, a large command graph and repeated entry points. At compact size the fixed canvas scales the entire interface and removes the workspace and system cards. [Desktop render](../../assets/desktop-preview.png) · [Connections render](https://github.com/sumerchoudhary78/hyprlume/blob/v0.5.1/assets/connections-preview.png) | Keep one visual anchor in the wallpaper. Give the command field one aligned surface and reflow the workspace and system cards on narrow displays. |
| GNOME’s system menu gives commonly used settings short paths. KDE’s guidance prioritizes meaningful status changes and uses on-screen feedback for system actions. [GNOME Quick Settings](https://release.gnome.org/43/) · [KDE status guidance](https://develop.kde.org/hig/status_changes/) | Keep connectivity, sound, notifications and session controls available from the bar. Show a recording stop control during recording and a distinct microphone-off icon when capture is off. |
| Apple’s material guidance uses translucency to establish depth and calls for restraint on text-heavy surfaces. Qt’s performance guide identifies blending, offscreen captures and per-pixel shader work as costs to measure. The older Hyprlume liquid desktop sample used 3.864% of one CPU core while the dashboard was visible. [Material guidance](https://developer.apple.com/design/human-interface-guidelines/materials) · [Qt Quick performance](https://doc.qt.io/qt-6/qtquick-performance.html) · [Historical sample](performance-liquid.json) | Use dense surfaces for lists and overlays. Limit wallpaper sampling to the command card when it fits without scrolling and effects are enabled. Use static system meters on Home. |
| WCAG’s contrast guidance gives a 4.5:1 target for regular text. Qt Quick provides focus scopes, accessible roles and accessible names for custom controls. [Contrast minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) · [Qt focus](https://doc.qt.io/qt-6/qtquick-input-focus.html) · [Qt accessibility](https://doc.qt.io/qt-6/qml-qtquick-accessible.html) | Use measurable foreground/background pairs, native buttons with visible keyboard focus, and labels for command, microphone and device states. High contrast removes optical glass. |

## Framework choice

| Framework | Fit |
| --- | --- |
| **Quickshell and Qt Quick Controls, selected** | The shell already uses Quickshell for per-output panels, Wayland layer ordering, space reservation, keyboard focus, Hyprland state and native services. Qt Quick Layouts, Controls and QML are sufficient for the new composition. The Linux preview exercises the production components. [PanelWindow](https://quickshell.org/docs/v0.3.1/types/Quickshell/PanelWindow/) · [Wayland layers](https://quickshell.org/docs/v0.3.0/types/Quickshell.Wayland/WlrLayershell/) |
| **AGS, Astal and GTK 4** | A workable native option with TypeScript and GTK layer shell. A migration would replace QML control code and carry new service integration, rendering, packaging and input qualification work. [AGS quick start](https://aylur.github.io/ags/guide/quick-start.html) · [GTK layer-shell requirements](https://aylur.github.io/ags/guide/install.html) |
| **Tauri and a web frontend** | A workable choice for a separate app window. Linux renders Tauri UIs in WebKitGTK. Hyprlume would still need native shell surfaces and a new bridge for session services. [Tauri WebViews](https://v2.tauri.app/reference/webview-versions/) |

## Layout and interaction

- **Bar.** Apps and Command have visible labels at standard widths. System
  controls have accessible names and hover labels. On tighter outputs the bar
  wraps its system controls into another row and reserves its actual height.
  Recording shows a red Stop recording button in the first row.
- **Home.** The focused workspace name, command entry and shortcuts lead the
  layout. The workspace list shows window counts and a visible current marker.
  CPU, memory and GPU are glanceable. System details disclose network rates,
  disk use, load, uptime, container state and CPU history. Home reflows below
  820 scaled pixels of usable content width and scrolls on short outputs.
- **Command.** Enter previews. A pending review shows the requested operations
  and requires a separate Run plan action. The command panel has scrollable
  content for long requests and keeps voice controls outside the scrolling
  area. The direct-run button stays explicit and visually secondary.
- **Utility panels.** Use a solid card, simple title and clear action states.
  Connections gives Wi-Fi and Bluetooth the same visual weight and uses tabs
  where the panes need to stack. Auth and session actions keep their existing
  separate confirmations and protocol paths.
- **Visual system.** Inter for interface text and JetBrains Mono for quantities
  and technical text where alignment helps. The base text sizes are 12, 14 and
  22 logical pixels for metadata, body and prominent labels. Card radii are
  16 and 20 pixels. A configurable action accent distinguishes primary actions.
  Muted body text at `#b5c7cf` reaches 8.55:1 on the panel’s lightest base
  color, `#1a2932`. The primary button text reaches 12.18:1 on cyan,
  8.53:1 on violet, and 12.56:1 on mint. [Theme tokens](../../shell/theme/Theme.qml)
- **Wallpaper.** Tidal Eclipse uses a single bright crescent and a quiet left
  half. Text-bearing panels use a dark material, and Home adds a dimming layer
  over custom backgrounds. High contrast and reduced effects preserve the
  layout and actions with simpler rendering. [Asset provenance](../../shell/assets/README.md)

## Maintain the design

Keep typography and control sizes in logical pixels. Vector icons and charts
use Qt’s curve renderer where available. Review the full-resolution preview
at 100% zoom. Keep animation to a short translation of the opening panel
and let reading-heavy surfaces settle. Profile frame timing on the target
GPU at the intended refresh rate. [Qt Quick shapes](https://doc.qt.io/qt-6/qml-qtquick-shapes-shape.html) ·
[Qt frame timing](https://doc.qt.io/qtcreator/creator-qml-performance-monitor.html)

Use the same QML controls for live and fixture views. Check desktop and narrow
layouts, reduced effects, the full supported scale range, keyboard traversal,
and busy, unavailable and review states. Run the existing native pointer and
keyboard checks when changing command or utility-panel entry points. The
reproduction commands are in the [desktop guide](../homescreen.md). Physical
display, GPU and session measurements belong in the [validation record](../validation.md).
