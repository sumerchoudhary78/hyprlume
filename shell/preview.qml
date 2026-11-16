import QtQuick
import Quickshell

ShellRoot {
    FloatingWindow {
        id: window
        visible: true
        implicitWidth: 780
        implicitHeight: panel.implicitHeight + 96
        color: "#101927"
        GlassPanel {
            id: panel
            anchors.centerIn: parent
            width: 620
            snapshot: ({phase: "review", message: "A clear plan. One action away.", transcript: "Move this window to workspace two.", actions: ["Move Firefox to workspace 2"], receipts: [], confirmation: 1, panel_open: true, voice: "off", workspace: 1, active_app: "firefox"})
        }
        Timer {
            interval: 600
            running: true
            onTriggered: panel.grabToImage(result => {
                if (!result.saveToFile(Quickshell.env("HYPRLUME_SCREENSHOT"))) console.error("screenshot failed")
                Qt.quit()
            })
        }
    }
}
