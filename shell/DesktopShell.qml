import QtQuick
import Quickshell
import Quickshell.Hyprland
import Quickshell.Wayland
import qs.components
import qs.modules.tray
import qs.modules.power
import qs.modules.audio
import qs.modules.command
import qs.modules.notifications
import qs.modules.connectivity
import qs.modules.home
import qs.modules.scene
import qs.modules.topbar
import qs.theme

// Composition root: one bar per monitor, plus the desktop dashboard on the
// focused monitor, plus mutually exclusive command and connectivity overlays.
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
        active: desktop.backend.online && desktop.backend.snapshot.panel_open && !desktop.backend.snapshot.desktop_panel && !desktop.backend.connectivity.panel_open && !desktop.backend.notifications.panel_open

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

    Loader {
        active: desktop.backend.online && !desktop.backend.snapshot.desktop_panel && desktop.backend.connectivity.panel_open && !desktop.backend.notifications.panel_open
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-connectivity"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            ConnectivityOverlay {
                anchors.fill: parent; backend: desktop.backend
                reducedEffects: desktop.state.reducedEffects
            }
        }
    }

    Loader {
        active: desktop.backend.online && desktop.backend.notifications.panel_open
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-notification-centre"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            NotificationOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.state.reducedEffects }
        }
    }
    Loader {
        active: desktop.backend.online && !desktop.backend.notifications.panel_open && !desktop.backend.notifications.dnd && desktop.backend.notifications.items.some(item => item.active && item.popup)
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors { top: true; right: true }
            margins { top: Theme.barZone + 12; right: 14 }
            implicitWidth: Math.min(390, desktop.focusedScreen ? desktop.focusedScreen.width - 28 : 390)
            implicitHeight: popupStack.implicitHeight + 18
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-notification-popups"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
            PopupStack {
                id: popupStack
                anchors.fill: parent; anchors.margins: 9
                availableHeight: desktop.focusedScreen ? desktop.focusedScreen.height - Theme.barZone - 30 : 800
                backend: desktop.backend; reducedEffects: desktop.state.reducedEffects
            }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "audio"
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-audio"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            AudioOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.state.reducedEffects }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "power"
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-power"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            PowerOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.state.reducedEffects }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "tray"
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-tray"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            TrayOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.state.reducedEffects }
        }
    }

}
