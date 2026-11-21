# Workspace atlas

The First Light desktop put workspace names behind a button. The atlas brings the arrangement of the windows themselves onto an empty workspace. A browser beside a narrow notes window is a different target from three tiled terminals, even before you read their titles. [Earlier desktop](first-light-desktop.webp) · [Atlas desktop](atlas-desktop.webp)

## Finding a window

Click a shape to focus that exact window. Click a workspace label or the background of its map to switch workspaces. The current workspace has a copper outline and a text label. The empty desktop shows as many maps as fit. Open the atlas from the bar or its own button to see every workspace over an application. [Expanded atlas](atlas-open.webp)

The expanded view initially focuses the current workspace. The arrow keys move between maps, Home and End jump to the ends, and Enter switches workspaces. The window list also lets you reach small or overlapping windows by title. Window shapes require a pointer target of at least 24 logical pixels in each dimension. Small shapes remain visible. [GNOME keyboard guidance](https://developer.gnome.org/hig/guidelines/keyboard.html) · [W3C target-size guidance](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)

The status bar keeps Apps and Command one click away, with recording and microphone stop controls visible while active. Quick Settings groups the device and session controls. [Quick Settings](atlas-controls.webp) · [KDE guidance on grouping controls](https://develop.kde.org/hig/simple_by_default/)

## Drawing the map

Hyprland supplies the workspace, application, title and rectangle for each window. Window rectangles are converted to the output's logical coordinates, clipped to its edges, and drawn at the output's aspect ratio. Floating windows appear above tiled windows. Entries with missing geometry remain available by title. Focusing a window uses Hyprlume's existing address-based command, which checks that the target still exists and that the compositor focused it. [Quickshell window API](https://quickshell.org/docs/v0.3.0/types/Quickshell.Hyprland/HyprlandToplevel/) · [Quickshell monitor API](https://quickshell.org/docs/v0.3.0/types/Quickshell.Hyprland/HyprlandMonitor/)

Quickshell exposes window rectangles through its last IPC snapshot. A single refresh runs every 1.8 seconds while a map and at least one window are present. Hiding every map stops that timer. The atlas uses metadata, so it needs no live screenshots or screen-sharing permission. The wallpaper surface accepts pointer events only over the controls on an empty workspace. [Quickshell refresh API](https://quickshell.org/docs/v0.3.0/types/Quickshell.Hyprland/Hyprland/)

Caldera is a cut-paper relief in petrol, teal and copper. The serif heading is Instrument Serif. App names and controls use the shell's normal text faces. The atlas also works with a custom wallpaper, high contrast and reduced effects. [Asset sources and licenses](../../shell/assets/README.md)

## Verification

The design previews use labeled sample data. Native pointer and keyboard checks cover choosing a window from a map and from the title list, returning to an empty workspace and using the compact layout. Geometry tests cover fractional scaling, rotation, clipping and negative monitor origins. [Run the checks](../homescreen.md)

The interaction demo captures two live terminal windows on headless Sway. A test adapter translates their positions and focus events into Hyprland's IPC format. Resizing the editor changes its footprint, clicking the other tile focuses the terminal through the Rust daemon, and closing the editor expands the remaining tile. GPU rendering on a physical Hyprland session still needs verification. [Watch the window switch](atlas-window-switch.webp)
