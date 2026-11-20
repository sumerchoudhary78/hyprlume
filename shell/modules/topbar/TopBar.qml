import QtQuick
import Quickshell.Wayland
import qs.components

ShellWindow {
    id: root
    surfaceName: "bar"
    required property var backend
    required property var clock

    anchors { top: true; left: true; right: true }
    implicitHeight: bar.implicitHeight
    exclusiveZone: implicitHeight
    WlrLayershell.layer: WlrLayer.Top

    BarContent {
        id: bar
        anchors.fill: parent
        backend: root.backend
        clock: root.clock
    }
}
