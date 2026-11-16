import QtQuick
import Quickshell
import Quickshell.Hyprland
import Quickshell.Wayland
import qs.components
import qs.modules.command
import qs.modules.home
import qs.modules.scene
import qs.modules.topbar
import qs.theme

// Composition root: one bar per monitor, plus the desktop dashboard on the
// focused monitor, plus the command panel overlay.
Item {
    id: desktop

    required property var backend
    required property var clock
    required property var metrics
    required property var workspaces
    required property var state

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

        delegate: Item {
            id: output
            required property var modelData
            WallpaperSurface { screen: output.modelData }
            TopBar {
                screen: output.modelData
                backend: desktop.backend
                workspaces: desktop.workspaces
                state: desktop.state
            }
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

            anchors {
                top: true
                bottom: true
                left: true
                right: true
            }

            exclusiveZone: 0
            color: "transparent"
            WlrLayershell.namespace: "hyprlume-command"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive

            CommandOverlay {
                anchors.fill: parent
                backend: desktop.backend
            }
        }
    }
}
