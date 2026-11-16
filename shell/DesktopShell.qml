import QtQuick
import Quickshell
import Quickshell.Hyprland
import Quickshell.Wayland
import qs.components
import qs.modules.home
import qs.modules.topbar
import qs.services
import qs.theme

// Composition root: one bar per monitor, plus the desktop dashboard on the
// focused monitor, plus the command panel overlay.
Item {
    id: desktop

    required property Backend backend
    required property Clock clock
    required property Metrics metrics
    required property Workspaces workspaces
    required property ShellState state

    // The dashboard belongs to the monitor holding the active workspace.
    // Falling back to the first screen keeps single-monitor sessions simple
    // and keeps the surface visible if Hyprland's monitor list is late.
    readonly property string focusedScreenName: {
        const monitor = Hyprland.focusedMonitor;
        if (monitor && monitor.name && monitor.name.length > 0)
            return monitor.name;
        return Quickshell.screens.length > 0 ? Quickshell.screens[0].name : "";
    }

    readonly property var focusedScreen: {
        const screens = Quickshell.screens;
        for (let i = 0; i < screens.length; i++) {
            if (screens[i].name === desktop.focusedScreenName)
                return screens[i];
        }
        return screens.length > 0 ? screens[0] : null;
    }

    Variants {
        model: Quickshell.screens

        delegate: TopBar {
            required property var modelData

            screen: modelData
            backend: desktop.backend
            workspaces: desktop.workspaces
            state: desktop.state
        }
    }

    Home {
        screen: desktop.focusedScreen
        backend: desktop.backend
        workspaces: desktop.workspaces
        metrics: desktop.metrics
        clock: desktop.clock
        state: desktop.state
    }

    Loader {
        active: desktop.backend.snapshot.panel_open

        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors.top: true
            margins.top: Theme.px(84)
            implicitWidth: desktop.focusedScreen
                ? Math.min(620, desktop.focusedScreen.width - 32)
                : 620
            implicitHeight: nativePanel.implicitHeight
            exclusiveZone: 0
            color: "transparent"
            WlrLayershell.namespace: "hyprlume-command"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive

            GlassPanel {
                id: nativePanel
                anchors.fill: parent
                snapshot: desktop.backend.snapshot
                connectionError: desktop.backend.error
                onRequest: operation => desktop.backend.send(operation)
                onDismissed: desktop.backend.send({ op: "toggle" })
                Component.onCompleted: focusInput()
            }
        }
    }
}
