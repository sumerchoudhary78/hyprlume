import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: backend
    property bool online: false
    property string error: ""
    property int sequence: 0
    property var snapshot: ({phase: "offline", message: "Start hyprlume daemon to connect", transcript: "", actions: [], receipts: [], confirmation: null, panel_open: false, voice: "off", workspace: null, active_app: ""})
    property var connectivity: ({panel_open:false,busy:false,message:"Connecting to desktop services",error:"",wifi:{available:false,enabled:false,hardware_enabled:false,interfaces:[],networks:[],error:""},bluetooth:{available:false,adapters:[],devices:[],error:""},prompt:null})
    property var notifications: ({available:false,error:"Connecting to notifications",panel_open:false,dnd:false,unread:0,items:[]})
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
                backend.send({op: "connectivity_subscribe"})
                backend.send({op: "notifications_subscribe"})
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
                    else if (data.event === "connectivity") backend.connectivity = data.state
                    else if (data.event === "notifications") backend.notifications = data.state
                    else if (data.result && data.result.phase) backend.snapshot = data.result
                    else if (data.result && data.result.wifi && data.result.bluetooth) backend.connectivity = data.result
                    else if (data.result && data.result.items && data.result.unread !== undefined) backend.notifications = data.result
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
