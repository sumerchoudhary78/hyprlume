import QtQuick
import Quickshell
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

// Ordinary-window host for the same live dashboard used by layer-shell.
FloatingWindow {
    id: window
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    required property var shellState
    title: "Hyprlume Preview"
    Component.onCompleted: if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_LIVE_PREVIEW_READY")
    visible: true
    implicitWidth: 1440
    implicitHeight: 810
    color: "#030914"
    Dashboard {
        anchors.fill: parent
        backend: window.backend; workspaces: window.workspaces; metrics: window.metrics; clock: window.clock; shellState: window.shellState
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.snapshot.panel_open && !window.backend.snapshot.desktop_panel && !window.backend.connectivity.panel_open && !window.backend.notifications.panel_open
        sourceComponent: CommandOverlay {
            anchors.fill: parent
            backend: window.backend
            reducedEffects: window.shellState.reducedEffects
        }
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && !window.backend.snapshot.desktop_panel && window.backend.connectivity.panel_open && !window.backend.notifications.panel_open
        sourceComponent: ConnectivityOverlay {
            anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects
        }
    }

    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.notifications.panel_open
        sourceComponent: NotificationOverlay { anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }
    PopupStack {
        anchors.right: parent.right; anchors.top: parent.top
        anchors.rightMargin: 16; anchors.topMargin: 88
        width: Math.min(380, parent.width - 32)
        availableHeight: parent.height - 106
        backend: window.backend; reducedEffects: window.shellState.reducedEffects
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.snapshot.desktop_panel === "audio"
        sourceComponent: AudioOverlay { anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.snapshot.desktop_panel === "power"
        sourceComponent: PowerOverlay { anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.snapshot.desktop_panel === "tray"
        sourceComponent: TrayOverlay { anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }

    Loader {
        anchors.fill: parent
        active: window.backend.online && ["launcher","switcher"].includes(window.backend.snapshot.desktop_panel)
        sourceComponent: LauncherOverlay { anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && ["clipboard","capture"].includes(window.backend.snapshot.desktop_panel)
        sourceComponent: ToolsOverlay { outputName:window.screen ? window.screen.name : ""; anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.snapshot.desktop_panel === "storage"
        sourceComponent: StorageOverlay { anchors.fill: parent; backend: window.backend; reducedEffects: window.shellState.reducedEffects }
    }
    Loader {
        anchors.fill:parent
        active: window.backend.online && window.backend.snapshot.desktop_panel === "settings"
        sourceComponent: SettingsOverlay {anchors.fill:parent;backend:window.backend;reducedEffects:window.shellState.reducedEffects}
    }
}
