import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland

Item {
    id: desktop
    required property Backend backend
    PanelWindow {
        anchors.top: true
        margins.top: 10
        implicitWidth: 286
        implicitHeight: 42
        exclusiveZone: 0
        color: "transparent"
        WlrLayershell.namespace: "hyprlume"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        Rectangle {
            anchors.fill: parent
            radius: 15
            color: "#f01b2839"
            border.width: 1
            border.color: "#53667b"
            RowLayout {
                anchors { fill: parent; leftMargin: 14; rightMargin: 8 }
                spacing: 10
                Text { text: "H"; color: "#a5e3d2"; font.pixelSize: 18; font.weight: Font.Medium }
                Text { text: !desktop.backend.online ? "Disconnected" : (desktop.backend.snapshot.voice === "off" ? "Microphone off" : (desktop.backend.snapshot.phase === "speaking" ? "Speaking · mic paused" : "Listening · " + desktop.backend.snapshot.voice)); color: "#e6eef5"; font.pixelSize: 12; Layout.fillWidth: true }
                LumeButton { text: "Open"; compact: true; onClicked: desktop.backend.send({op: "toggle"}) }
            }
        }
    }
    Loader {
        active: desktop.backend.snapshot.panel_open
        sourceComponent: PanelWindow {
            anchors.top: true
            margins.top: 72
            implicitWidth: Math.min(620, screen.width - 32)
            implicitHeight: nativePanel.implicitHeight
            exclusiveZone: 0
            color: "transparent"
            WlrLayershell.namespace: "hyprlume"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            GlassPanel {
                id: nativePanel
                anchors.fill: parent
                snapshot: desktop.backend.snapshot
                connectionError: desktop.backend.error
                onRequest: operation => desktop.backend.send(operation)
                onDismissed: desktop.backend.send({op: "toggle"})
                Component.onCompleted: focusInput()
            }
        }
    }
}
