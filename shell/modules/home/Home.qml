import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.components

ShellWindow {
    id: root
    surfaceName: "home"
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    required property var shellState
    anchors { top: true; bottom: true; left: true; right: true }
    exclusiveZone: 0
    WlrLayershell.layer: WlrLayer.Bottom
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.OnDemand
    visible: root.workspaces.desktopVisible
    Dashboard {
        anchors.fill: parent
        backend: root.backend; workspaces: root.workspaces; metrics: root.metrics; clock: root.clock; shellState: root.shellState
        showTopBar: false
        wallpaperHeight: root.screen ? root.screen.height : height
    }
}
