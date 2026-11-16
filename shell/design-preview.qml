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
        readonly property bool effects: Quickshell.env("HYPRLUME_EFFECTS_QA") === "1"
        visible: true
        implicitWidth: Number(Quickshell.env("HYPRLUME_PREVIEW_WIDTH")) || 1680
        implicitHeight: Number(Quickshell.env("HYPRLUME_PREVIEW_HEIGHT")) || 945
        color: "#030914"
        Loader {
            id: scene
            anchors.fill: parent
            sourceComponent: window.effects ? effectsComponent : dashboardComponent
        }
        Component {
            id: dashboardComponent
            Dashboard {
                backend: previewData.backend; workspaces: previewData.workspaces; metrics: previewData.metrics; clock: previewData.clock; state: previewData.state
                presentation: true
                animationsActive: window.visible && Window.visibility !== Window.Minimized
            }
        }
        Component { id: effectsComponent; EffectsPreview {} }
        Loader {
            anchors.fill: parent
            active: previewData.backend.snapshot.panel_open && !window.effects
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
            interval: window.effects ? 3600 : 2400
            running: Quickshell.env("HYPRLUME_SCREENSHOT").length > 0
            onTriggered: scene.item.grabToImage(result => {
                const ok = result.saveToFile(Quickshell.env("HYPRLUME_SCREENSHOT"));
                console.log(ok ? "HYPRLUME_RENDER_PASS" : "HYPRLUME_RENDER_FAIL");
                Qt.quit();
            })
        }
    }
}
