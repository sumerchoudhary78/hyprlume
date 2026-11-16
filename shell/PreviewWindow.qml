import QtQuick
import Quickshell
import qs.services

FloatingWindow {
    id: window
    required property Backend backend
    visible: true
    implicitWidth: 760
    implicitHeight: Math.max(620, previewPanel.implicitHeight + 80)
    color: "#101927"
    GlassPanel {
        id: previewPanel
        anchors.centerIn: parent
        width: Math.min(parent.width - 48, implicitWidth)
        snapshot: window.backend.snapshot
        connectionError: window.backend.error
        onRequest: operation => window.backend.send(operation)
        onDismissed: Qt.quit()
    }
}
