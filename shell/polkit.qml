import QtQuick
import Quickshell
import Quickshell.Hyprland
import Quickshell.Wayland
import qs.modules.polkit

// Standalone process. Neither the live shell nor its preview loads this entry.
ShellRoot {
    id: root
    readonly property string binary: Quickshell.env("HYPRLUME_POLKIT_BIN") || ""
    readonly property bool enabled: binary.length > 0 && Quickshell.env("HYPRLUME_PREVIEW") !== "1"
    property var promptScreen: null

    Loader {
        active: root.enabled
        sourceComponent: Scope {
            LockGate { id: login }
            AgentChannel {
                id: agentChannel
                command: [root.binary, "polkit-worker"]
                available: login.available
                onRegistered: console.info("Hyprlume Polkit agent registered")
                onWorkerExited: status => Qt.exit(status)
                onRequestChanged: {
                    if (request) {
                        const monitor = Hyprland.focusedMonitor?.name;
                        root.promptScreen = Quickshell.screens.find(screen => screen.name === monitor) || Quickshell.screens[0] || null;
                    }
                }
            }
            Loader {
                active: agentChannel.request !== null && login.available
                sourceComponent: PanelWindow {
                    screen: root.promptScreen
                    anchors { top: true; bottom: true; left: true; right: true }
                    exclusionMode: ExclusionMode.Ignore
                    color: "transparent"
                    WlrLayershell.namespace: "hyprlume-polkit"
                    WlrLayershell.layer: WlrLayer.Overlay
                    WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
                    AuthDialog { anchors.fill: parent; agent: agentChannel }
                }
            }
        }
    }
    Component.onCompleted: {
        if (!enabled) {
            console.error("Launch the Polkit dialog with hyprlume polkit in a Wayland session.");
            Qt.exit(1);
        }
    }
}
