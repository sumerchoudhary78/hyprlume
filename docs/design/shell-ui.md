# Desktop design

In the v0.6.0 desktop render, the active workspace name appears in the heading, in the workspace list and again in the latest command receipt. The command editor, starter commands, agent status and three hardware meters are visible before the user has a task. The current design gives that space back to the wallpaper. The bar names the active workspace, offers Apps and Command, and makes notifications and system controls available over a tiled window. [v0.6.0 desktop render](https://github.com/sumerchoudhary78/hyprlume/blob/v0.6.0/docs/design/shell-preview.webp)

## What informed the changes

| Design question | Evidence and decision |
| --- | --- |
| What deserves a permanent place on screen? | GNOME puts windows and applications in an overview and uses the top bar for access to that overview, notifications and system settings. Hyprlume keeps its distinctive command entry one click away. The workspace button opens a sheet that closes as soon as the user chooses a workspace. [GNOME desktop overview](https://help.gnome.org/gnome-help/shell-introduction.html) |
| Where should the hardware controls go? | KDE recommends grouping related controls and prioritizing frequent actions. The previous bar exposed nine system buttons at once. Quick Settings now offers output volume and mute directly, alongside named destinations for Connections, Sound, Desktop tools and the app tray. Reaching a detailed device panel costs a second click. [KDE guidance on simplifying controls](https://develop.kde.org/hig/simple_by_default/) |
| Which state changes need attention? | KDE recommends using color selectively and pairing it with a shape or text. Active voice capture and screen recording get dedicated stop controls in the bar. The current workspace has a check mark, and Quick Settings spells out the microphone and Do Not Disturb states. [KDE status guidance](https://develop.kde.org/hig/status_changes/) |
| What happens when the monitor gets narrower? | GNOME recommends full keyboard operation. W3C describes a minimum pointer target of 24 CSS pixels or equivalent spacing. Using that as a reference, the smallest Qt bar buttons measure 34 by 40 logical pixels at the default scale. Labels and optional readings collapse first. The menus scroll and keyboard focus reveals controls below the fold. [GNOME keyboard guidance](https://developer.gnome.org/hig/guidelines/keyboard.html) · [W3C target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) |

## Materials and rendering

First Light is a locally bundled alpine landscape with warm highlights and a dark foreground. People who have selected a wallpaper continue to see their own image. The bar and sheets provide their own dark background for text. High contrast uses opaque surfaces and brighter outlines. Inter is the interface face, and JetBrains Mono is used for numeric readouts. [Asset origins](../../shell/assets/README.md) · [Contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html)

Quickshell provides per-output panels, Wayland keyboard focus, live workspace objects and native audio. The idle desktop owns a background surface and a 40-logical-pixel bar per output at default scale. Opening a menu creates one overlay on the selected output. Only an open menu over an empty workspace can sample the wallpaper for glass, and opening Overview starts the optional metrics sampler. [Quickshell monitor API](https://quickshell.org/docs/v0.3.0/types/Quickshell.Hyprland/HyprlandMonitor/) · [Quickshell exclusion modes](https://quickshell.org/docs/v0.3.0/types/Quickshell/ExclusionMode/)

## Reviewing the result

These captures use the production QML with sample session data. [Idle desktop](first-light-desktop.webp) · [Overview](first-light-overview.webp) · [Quick Settings](first-light-controls.webp)

The desktop guide includes commands for opening each view, rendering at different scales and running the keyboard and pointer checks. [Desktop guide](../homescreen.md)

The bar layout test covers 17 widths at two shell scales, with a long workspace name, a 12-hour clock, an active microphone and a screen recording. A two-output headless Wayland session also verified that opening Overview over a tiled app preserves the bar reservation and that dismissing it stops the system sampler.
