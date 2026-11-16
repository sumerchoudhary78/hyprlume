import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.components
import qs.theme

ShellWindow {
    id: root
    surfaceName: "bar"
    required property var backend
    required property var workspaces
    required property var state
    readonly property real sceneScale: Math.min(width / 1680, (screen ? screen.height : 945) / 945)
    anchors { top: true; left: true; right: true }
    implicitHeight: Math.ceil(63 * sceneScale)
    exclusiveZone: implicitHeight
    WlrLayershell.layer: WlrLayer.Top
    BarContent {
        x: Math.max(0, root.width - width * scale)
        width: Math.min(1180, root.width / scale); height: 60
        scale: root.sceneScale
        transformOrigin: Item.TopLeft
        backend: root.backend; state: root.state
    }
}
