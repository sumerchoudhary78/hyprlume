import QtQuick
import Quickshell
import Quickshell.Io

// This connection only reads the lock topic. PAM never uses the daemon socket.
Scope {
    id: root

    property string path: Quickshell.env("HYPRLUME_SOCKET") ||
        (Quickshell.env("XDG_RUNTIME_DIR") + "/hyprlume/control.sock")
    property bool unlocked: false
    readonly property bool available: connection.connected && unlocked

    Socket {
        id: connection
        path: root.path
        connected: false
        onConnectedChanged: {
            root.unlocked = false;
            if (connected) {
                retry.stop();
                write('{"id":1,"op":"lock_subscribe"}\n');
                flush();
            } else {
                retry.restart();
            }
        }
        onError: {
            root.unlocked = false;
            retry.restart();
        }
        parser: SplitParser {
            onRead: line => {
                try {
                    const packet = JSON.parse(line);
                    const state = packet.event === "locking" ? packet.state : packet.id === 1 ? packet.result : null;
                    root.unlocked = !!state && state.active === false;
                } catch (_) {
                    root.unlocked = false;
                }
            }
        }
    }
    Timer {
        id: retry
        interval: 2000
        onTriggered: {
            connection.connected = false;
            connection.connected = true;
        }
    }
    Component.onCompleted: connection.connected = true
}
