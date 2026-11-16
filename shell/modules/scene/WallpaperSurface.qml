import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.components

// The room persists behind ordinary application windows. This surface never
// receives pointer or keyboard input and reserves no desktop space.
ShellWindow {
    surfaceName: "wallpaper"
    anchors { top: true; bottom: true; left: true; right: true }
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Background
    mask: Region {}
    Wallpaper { anchors.fill: parent }
}
