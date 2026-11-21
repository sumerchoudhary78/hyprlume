import QtQuick
import Quickshell
import Quickshell.Hyprland
import Quickshell.Wayland
import qs.components
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
import qs.modules.home
import qs.modules.scene
import qs.modules.topbar
import qs.theme

// Persistent surfaces belong to each output. Menus and service panels load on demand.
Item {
    id: desktop

    required property var backend
    required property var clock
    required property var metrics
    required property var workspaces
    required property var shellState

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

    readonly property var panelScreen: shellState.panelScreen ?? focusedScreen

    readonly property var feedback: backend.online && !backend.locking.active ? backend.snapshot.feedback : null
    property bool feedbackVisible: false
    onFeedbackChanged: {
        feedbackVisible = !!feedback && Date.now() - feedback.issued_at < 1700;
        if (feedbackVisible) feedbackTimeout.restart();
    }
    Timer { id: feedbackTimeout; interval: 1700; onTriggered: desktop.feedbackVisible = false }
    Loader {
        active: desktop.feedbackVisible
        sourceComponent: PanelWindow {
            screen: desktop.focusedScreen
            anchors.bottom: true
            margins.bottom: 52
            implicitWidth: 315
            implicitHeight: 60
            exclusiveZone: 0
            color: "transparent"
            mask: Region {}
            WlrLayershell.namespace: "hyprlume-system-feedback"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
            SystemFeedback {
                anchors.fill: parent
                feedback: desktop.feedback
                reducedEffects: desktop.shellState.reducedEffects
            }
        }
    }

    Variants {
        model: Quickshell.screens

        delegate: Item {
            id: output
            required property var modelData
            WallpaperSurface { screen: output.modelData; workspaces: desktop.workspaces; shellState: desktop.shellState }
            TopBar {
                screen: output.modelData
                backend: desktop.backend
                clock: desktop.clock
                shellState: desktop.shellState
                workspaceLabel: desktop.workspaces.label(desktop.workspaces.activeOn(output.modelData))
            }
        }
    }

    Loader {
        active: !!desktop.shellState.menu
        sourceComponent: ShellWindow {
            id: menuWindow
            surfaceName: "desktop-menu"
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            DesktopMenu {
                anchors.fill: parent
                backend: desktop.backend; shellState: desktop.shellState
                workspaces: desktop.workspaces; metrics: desktop.metrics
                currentWorkspace: menuWindow.screen ? desktop.workspaces.activeOn(menuWindow.screen) : null
                wallpaperHeight: menuWindow.screen ? menuWindow.screen.height : height
            }
        }
    }

    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.panel_open && !desktop.backend.snapshot.desktop_panel && !desktop.backend.connectivity.panel_open && !desktop.backend.notifications.panel_open

        sourceComponent: PanelWindow {
            screen: desktop.panelScreen

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
                reducedEffects: desktop.shellState.reducedEffects
            }
        }
    }

    Loader {
        active: desktop.backend.online && !desktop.backend.snapshot.desktop_panel && desktop.backend.connectivity.panel_open && !desktop.backend.notifications.panel_open
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-connectivity"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            ConnectivityOverlay {
                anchors.fill: parent; backend: desktop.backend
                reducedEffects: desktop.shellState.reducedEffects
            }
        }
    }

    Loader {
        active: desktop.backend.online && desktop.backend.notifications.panel_open
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-notification-centre"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            NotificationOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects }
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
                backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects
            }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "storage"
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-storage"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            StorageOverlay {
                anchors.fill: parent
                backend: desktop.backend
                reducedEffects: desktop.shellState.reducedEffects
            }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "audio"
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-audio"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            AudioOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "power"
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-power"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            PowerOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects }
        }
    }
    Loader {
        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "tray"
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-tray"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            TrayOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects }
        }
    }

    Loader {
        active: desktop.backend.online && ["launcher","switcher"].includes(desktop.backend.snapshot.desktop_panel)
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-launcher"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            LauncherOverlay { anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects }
        }
    }
    Loader {

        active: desktop.backend.online && ["clipboard","capture"].includes(desktop.backend.snapshot.desktop_panel)
        sourceComponent: PanelWindow {
            screen: desktop.panelScreen
            anchors { top: true; bottom: true; left: true; right: true }
            exclusiveZone: 0; color: "transparent"
            WlrLayershell.namespace: "hyprlume-tools"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
            ToolsOverlay { outputName:desktop.panelScreen?.name ?? ""; anchors.fill: parent; backend: desktop.backend; reducedEffects: desktop.shellState.reducedEffects }
        }
    }
    Loader {

        active: desktop.backend.online && desktop.backend.snapshot.desktop_panel === "settings"
        sourceComponent: PanelWindow {
            screen:desktop.panelScreen
            anchors {top:true;bottom:true;left:true;right:true}
            exclusiveZone:0;color:"transparent"
            WlrLayershell.namespace:"hyprlume-settings"
            WlrLayershell.layer:WlrLayer.Overlay
            WlrLayershell.keyboardFocus:settingsSheet.fileDialogVisible ? WlrKeyboardFocus.None : WlrKeyboardFocus.Exclusive
            SettingsOverlay {id:settingsSheet;anchors.fill:parent;backend:desktop.backend;reducedEffects:desktop.shellState.reducedEffects}
        }
    }
}
