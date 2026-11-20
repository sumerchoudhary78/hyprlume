import QtQuick
import Quickshell
import qs.fixtures
import qs.theme
import qs.modules.home
import qs.modules.settings
import qs.modules.storage
import qs.modules.tools
import qs.modules.launcher
import qs.modules.tray
import qs.modules.power
import qs.modules.audio
import qs.modules.command
import qs.modules.notifications
import qs.modules.connectivity

ShellRoot {
    PreviewData { id: previewData }
    Binding {target:Theme;property:"scale";value:(Number(Quickshell.env("HYPRLUME_PREVIEW_SCALE")) || 1) * previewData.backend.settings.values.preferences.appearance.scale}
    Binding {target:Theme;property:"accentName";value:previewData.backend.settings.values.preferences.appearance.accent}
    Binding { target: Theme; property: "highContrast"; value: Quickshell.env("HYPRLUME_HIGH_CONTRAST") === "1" || previewData.backend.settings.values.preferences.appearance.high_contrast }
    Binding { target: Theme; property: "wallpaperPath"; value: previewData.backend.settings.values.preferences.desktop.wallpaper }
    Binding { target: Theme; property: "glassRefraction"; value: Theme.highContrast ? 0 : previewData.backend.settings.values.preferences.appearance.refraction }
    FloatingWindow {
        id: window
        title: "Hyprlume Design Preview"
        readonly property bool effects: Quickshell.env("HYPRLUME_EFFECTS_QA") === "1"
        visible: true
        implicitWidth: Number(Quickshell.env("HYPRLUME_PREVIEW_WIDTH")) || 1680
        implicitHeight: Number(Quickshell.env("HYPRLUME_PREVIEW_HEIGHT")) || 945
        color: "#030914"
        Item {
            id: canvas
            anchors.fill: parent
            Loader {
                id: scene
                anchors.fill: parent
                sourceComponent: window.effects ? effectsComponent : dashboardComponent
            }
            Component {
                id: dashboardComponent
                Dashboard {
                    backend: previewData.backend; workspaces: previewData.workspaces; metrics: previewData.metrics; clock: previewData.clock; shellState: previewData.state
                    presentation: true
                }
            }
            Component { id: effectsComponent; EffectsPreview {} }
            Loader {
                anchors.fill: parent
                active: previewData.backend.snapshot.desktop_panel === "storage" && !window.effects
                sourceComponent: StorageOverlay { anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
            }
            Loader {
                anchors.fill: parent
                active: previewData.backend.snapshot.panel_open && !previewData.backend.snapshot.desktop_panel && !previewData.backend.connectivity.panel_open && !previewData.backend.notifications.panel_open && !window.effects
                sourceComponent: CommandOverlay {
                    anchors.fill: parent
                    backend: previewData.backend
                    reducedEffects: previewData.state.reducedEffects
                }
            }
            Loader {
                anchors.fill: parent
                active: !previewData.backend.snapshot.desktop_panel && previewData.backend.connectivity.panel_open && !previewData.backend.notifications.panel_open && !window.effects
                sourceComponent: ConnectivityOverlay {
                    anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects
                }
            }

            Loader {
                anchors.fill: parent
                active: previewData.backend.notifications.panel_open
                sourceComponent: NotificationOverlay { anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
            }
            PopupStack {
                anchors.right: parent.right; anchors.top: parent.top
                anchors.rightMargin: 16; anchors.topMargin: 88
                width: Math.min(380, parent.width - 32)
                availableHeight: parent.height - 106
                backend: previewData.backend; reducedEffects: previewData.state.reducedEffects
            }
            Loader {
                anchors.fill: parent
                active: previewData.backend.online && previewData.backend.snapshot.desktop_panel === "audio"
                sourceComponent: AudioOverlay { anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
            }
            Loader {
                anchors.fill: parent
                active: previewData.backend.online && previewData.backend.snapshot.desktop_panel === "power"
                sourceComponent: PowerOverlay { anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
            }
            Loader {
                anchors.fill: parent
                active: previewData.backend.online && previewData.backend.snapshot.desktop_panel === "tray"
                sourceComponent: TrayOverlay { anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
            }
    Loader {
        anchors.fill: parent
        active: previewData.backend.online && ["launcher","switcher"].includes(previewData.backend.snapshot.desktop_panel)
        sourceComponent: LauncherOverlay { anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
    }
    Loader {
        anchors.fill: parent
        active: previewData.backend.online && ["clipboard","capture"].includes(previewData.backend.snapshot.desktop_panel)
        sourceComponent: ToolsOverlay { outputName:"PREVIEW-1"; anchors.fill: parent; backend: previewData.backend; reducedEffects: previewData.state.reducedEffects }
    }
    Loader {
        anchors.fill:parent
        active: previewData.backend.online && previewData.backend.snapshot.desktop_panel === "settings"
        sourceComponent: SettingsOverlay {anchors.fill:parent;backend:previewData.backend;reducedEffects:previewData.state.reducedEffects}
    }
            ControlProbe {
                targetItem: window.contentItem
                backend: previewData.backend
                enabled: Quickshell.env("HYPRLUME_QA") === "1"
            }
            Timer {
                interval: window.effects ? 3600 : 2400
                running: Quickshell.env("HYPRLUME_SCREENSHOT").length > 0
                onTriggered: {
                    const started = canvas.grabToImage(result => {
                        const ok = result.saveToFile(Quickshell.env("HYPRLUME_SCREENSHOT"));
                        console.log(ok ? "HYPRLUME_RENDER_PASS" : "HYPRLUME_RENDER_FAIL");
                        Qt.quit();
                    });
                    if (!started) { console.error("HYPRLUME_RENDER_FAIL capture did not start"); Qt.exit(1); }
                }
            }
        }
    }
}
