import QtQuick
import Quickshell
import qs.modules.home
import qs.modules.command
import qs.modules.connectivity

// Ordinary-window host for the same live dashboard used by layer-shell.
FloatingWindow {
    id: window
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    required property var state
    visible: true
    implicitWidth: 1440
    implicitHeight: 810
    color: "#030914"
    Dashboard {
        anchors.fill: parent
        animationsActive: window.visible && Window.visibility !== Window.Minimized
        backend: window.backend; workspaces: window.workspaces; metrics: window.metrics; clock: window.clock; state: window.state
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.snapshot.panel_open && !window.backend.connectivity.panel_open
        sourceComponent: CommandOverlay {
            anchors.fill: parent
            backend: window.backend
        }
    }
    Loader {
        anchors.fill: parent
        active: window.backend.online && window.backend.connectivity.panel_open
        sourceComponent: ConnectivityOverlay {
            anchors.fill: parent; backend: window.backend; reducedEffects: window.state.reducedEffects
        }
    }
}
