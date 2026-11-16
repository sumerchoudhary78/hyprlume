import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// Local runtime information only; no invented cloud server or capacity.
GlassSurface {
    id: root
    required property var metrics
    readonly property bool hasRuntime: metrics.containerRuntime.length > 0
    readonly property bool running: metrics.containerCount > 0
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 14; spacing: 5
        PanelHeader {
            Layout.fillWidth: true
            title: root.running ? root.metrics.containerName : "Local Runtime"
            iconName: "package-x-generic"; accent: Theme.cyan
            Text { text: root.running ? "● Running" : root.hasRuntime ? "● Idle" : "● Not installed"; color: root.running ? Theme.mint : Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 10 }
        }
        RowLayout {
            Layout.fillWidth: true; spacing: 10
            Icon { name: "cpu"; size: 15; color: Theme.inkDim }
            Text { text: root.running ? root.metrics.containerCpu.toFixed(1) + "% CPU" : "No active containers"; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11 }
            Item { Layout.fillWidth: true }
            Text { visible: root.running; text: root.metrics.containerMem; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11 }
        }
        Sparkline { Layout.fillWidth: true; Layout.fillHeight: true; Layout.minimumHeight: 25; values: root.metrics.cpuHistory; accent: Theme.cyan }
        RowLayout {
            Layout.fillWidth: true
            Text { text: root.hasRuntime ? root.metrics.containerRuntime + " · local" : "docker / podman"; color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 9 }
            Item { Layout.fillWidth: true }
            Text { text: "HOST CPU TREND"; color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 8 }
        }
    }
}
