import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: backend
    property bool online: false
    property string error: ""
    property int sequence: 0
    property var snapshot: ({phase: "offline", message: "Start hyprlume daemon to connect", transcript: "", actions: [], receipts: [], confirmation: null, panel_open: false, voice: "off", workspace: null, active_app: ""})
    readonly property string socketPath: Quickshell.env("HYPRLUME_SOCKET") || (Quickshell.env("XDG_RUNTIME_DIR") + "/hyprlume/control.sock")

    function send(operation) {
        if (!socket.connected) {
            error = "Daemon disconnected. Start hyprlume daemon in this session."
            return
        }
        error = ""
        operation.id = ++sequence
        socket.write(JSON.stringify(operation) + "\n")
        socket.flush()
    }

    Socket {
        id: socket
        path: backend.socketPath
        connected: false
        onConnectedChanged: {
            backend.online = connected
            if (connected) {
                reconnect.stop()
                reconnect.interval = 500
                backend.send({op: "subscribe"})
            } else {
                reconnect.restart()
            }
        }
        onError: {
            backend.online = false
            reconnect.restart()
        }
        parser: SplitParser {
            onRead: message => {
                try {
                    const data = JSON.parse(message)
                    if (data.event === "state") backend.snapshot = data.state
                    else if (data.result && data.result.phase) backend.snapshot = data.result
                    else if (data.error) backend.error = data.error
                } catch (error) {
                    backend.error = "Invalid daemon response"
                }
            }
        }
    }
    Timer {
        id: reconnect
        interval: 500
        repeat: false
        onTriggered: {
            socket.connected = false
            socket.connected = true
            interval = Math.min(interval * 2, 10000)
        }
    }
    Component.onCompleted: socket.connected = true
}
