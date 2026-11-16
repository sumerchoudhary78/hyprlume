import QtQuick
import Quickshell
import qs.fixtures
import qs.modules.home
import qs.modules.command

ShellRoot {
    PreviewData { id: previewData }
    FloatingWindow {
        id: window
        title: "Hyprlume Design Preview"
        visible: true
        implicitWidth: Number(Quickshell.env("HYPRLUME_PREVIEW_WIDTH")) || 1680
        implicitHeight: Number(Quickshell.env("HYPRLUME_PREVIEW_HEIGHT")) || 945
        color: "#030914"
        Dashboard {
            id: dashboard
            anchors.fill: parent
            backend: previewData.backend; workspaces: previewData.workspaces; metrics: previewData.metrics; clock: previewData.clock; state: previewData.state
            presentation: true
        }
        Loader {
            anchors.fill: parent
            active: previewData.backend.snapshot.panel_open
            sourceComponent: CommandOverlay {
                anchors.fill: parent
                backend: previewData.backend
            }
        }
        ControlProbe {
            targetItem: window.contentItem
            backend: previewData.backend
            enabled: Quickshell.env("HYPRLUME_QA") === "1"
        }
        Timer {
            interval: 2400
            running: Quickshell.env("HYPRLUME_SCREENSHOT").length > 0
            onTriggered: dashboard.grabToImage(result => {
                const ok = result.saveToFile(Quickshell.env("HYPRLUME_SCREENSHOT"));
                console.log(ok ? "HYPRLUME_RENDER_PASS" : "HYPRLUME_RENDER_FAIL");
                Qt.quit();
            })
        }
    }
}
