import QtQuick
import Quickshell.Wayland
import qs.components

ShellWindow {
    id: root
    surfaceName: "bar"
    required property var backend
    required property var clock
    required property var shellState
    required property string workspaceLabel

    anchors { top: true; left: true; right: true }
    implicitHeight: bar.implicitHeight
    exclusiveZone: implicitHeight
    WlrLayershell.layer: WlrLayer.Top

    BarContent {
        id: bar
        anchors.fill: parent
        backend: root.backend
        clock: root.clock
        shellState: root.shellState
        outputScreen: root.screen
        workspaceLabel: root.workspaceLabel
    }
}
