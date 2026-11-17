import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: backend
    property bool online: false
    property string error: ""
    property int sequence: 0
    property var audio: null
    property var tray: null
    property var launcher: null
    property var snapshot: ({phase: "offline", message: "Start hyprlume daemon to connect", transcript: "", actions: [], receipts: [], confirmation: null, panel_open: false, desktop_panel:null, voice: "off", workspace: null, active_app: ""})
    property var connectivity: ({panel_open:false,busy:false,message:"Connecting to desktop services",error:"",wifi:{available:false,enabled:false,hardware_enabled:false,interfaces:[],networks:[],error:""},bluetooth:{available:false,adapters:[],devices:[],error:""},prompt:null})
    property var notifications: ({available:false,error:"Connecting to notifications",panel_open:false,dnd:false,unread:0,items:[]})
    property var clipboard: ({enabled:false,available:false,items:[],error:""})
    property var capture: ({screenshot_available:false,recording_available:false,region_available:false,busy:false,recording:false,output:"",message:"Connecting to capture services",error:""})
    signal response(int id, bool ok, var result, string error)
    property var settings: ({revision:0,values:{preferences:{appearance:{reduced_effects:false,accent:"cyan",scale:1,refraction:16,metrics:true},shortcuts:{launcher:"SUPER+SPACE",windows:"SUPER+TAB",tools:"SUPER+V",lock:"SUPER+L"},services:{clipboard_history:false,notifications_dnd:false}},voice:{asr_dir:"",kws_dir:"",tts_dir:"",keywords_file:"",threads:2,execute_commands:false,speak_results:true},llm:{endpoint:null,model:"local",timeout_seconds:20}},path:"",restart_required:false,message:"Connecting to settings",hyprlang:"",lua:""})
    property var power: ({busy:false,message:"Connecting to power services",error:"",prompt:null,battery:{available:false,present:false,percentage:null,state:"unknown",seconds_remaining:null,error:""},backlights:[],profiles:{available:false,active:null,choices:[],degraded:"",error:""},session:{available:false,id:"",suspend:"no",logout:"no",reboot:"no",power_off:"no",error:""}})
    readonly property string socketPath: Quickshell.env("HYPRLUME_SOCKET") || (Quickshell.env("XDG_RUNTIME_DIR") + "/hyprlume/control.sock")

    function updatePower(next) {
        // Keep controls/focus alive when a battery or busy update carries the
        // same device list. Actual device changes still replace that model.
        if (JSON.stringify(next.backlights) === JSON.stringify(power.backlights)) next.backlights = power.backlights;
        if (JSON.stringify(next.profiles.choices) === JSON.stringify(power.profiles.choices)) next.profiles.choices = power.profiles.choices;
        power = next;
    }

    function send(operation) {
        if (!socket.connected) {
            error = "Daemon disconnected. Start hyprlume daemon in this session."
            return 0
        }
        error = ""
        operation.id = ++sequence
        socket.write(JSON.stringify(operation) + "\n")
        socket.flush()
        return operation.id
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
                backend.send({op: "power_subscribe"})
                backend.send({op:"clipboard_subscribe"})
                backend.send({op:"capture_subscribe"})
                backend.send({op:"settings_subscribe"})
            } else {
                if (backend.launcher && backend.launcher.busy) backend.launcher.finish(false, "The desktop service disconnected");
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
                    if (data.id !== undefined && data.id !== null) backend.response(data.id, !data.error, data.result || null, data.error || "");
                    if (backend.launcher && backend.launcher.pending && data.id === backend.launcher.pending) backend.launcher.finish(!data.error, data.error || (data.result && data.result.message));
                    if (data.event === "state") backend.snapshot = data.state
                    else if (data.event === "connectivity") backend.connectivity = data.state
                    else if (data.event === "notifications") backend.notifications = data.state
                    else if (data.event === "settings") backend.settings = data.state
                    else if (data.event === "clipboard") backend.clipboard = data.state
                    else if (data.event === "capture") backend.capture = data.state
                    else if (data.event === "power") backend.updatePower(data.state)
                    else if (data.result && data.result.values && data.result.revision !== undefined) backend.settings = data.result
                    else if (data.result && data.result.enabled !== undefined && data.result.items) backend.clipboard = data.result
                    else if (data.result && data.result.screenshot_available !== undefined) backend.capture = data.result
                    else if (data.result && data.result.phase) backend.snapshot = data.result
                    else if (data.result && data.result.wifi && data.result.bluetooth) backend.connectivity = data.result
                    else if (data.result && data.result.items && data.result.unread !== undefined) backend.notifications = data.result
                    else if (data.result && data.result.battery && data.result.session) backend.updatePower(data.result)
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
