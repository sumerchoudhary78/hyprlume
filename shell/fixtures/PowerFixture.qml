import QtQuick
import Quickshell

// The normal shell never loads this service. No native power API is imported.
QtObject {
    id: root
    property bool active: false
    property int generation: 0
    property int sequence: 50
    property var queued: null
    property int queuedGeneration: 0
    property var snapshot: ({
        busy:false,message:"Your power, in balance.",error:"",prompt:null,
        battery:{available:true,present:true,percentage:72,state:"discharging",seconds_remaining:16440,error:""},
        backlights:[{id:"sample-panel:1:2:1000",name:"Observatory display",percent:70,value:700,max:1000}],
        profiles:{available:true,active:"balanced",choices:["power-saver","balanced","performance"],degraded:"",error:""},
        session:{available:true,id:"7",suspend:"yes",logout:"yes",reboot:"challenge",power_off:"yes",error:""}
    })
    onSnapshotChanged: if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"power",state:snapshot}));
    onActiveChanged: {
        generation++;
        if (!active && snapshot.prompt) snapshot = Object.assign({}, snapshot, {prompt:null});
    }
    function send(request) {
        if (request.action === "cancel") {
            if (snapshot.prompt && request.token === snapshot.prompt.token) snapshot = Object.assign({}, snapshot, {prompt:null,message:"Session action cancelled"});
            return;
        }
        if (snapshot.busy) return;
        if (request.action === "confirm" && (!snapshot.prompt || request.token !== snapshot.prompt.token || Date.now() >= snapshot.prompt.expires_at)) return;
        queued = request; queuedGeneration = generation;
        snapshot = Object.assign({}, snapshot, {busy:true,error:"",prompt:request.action === "confirm" ? null : snapshot.prompt});
        delay.restart();
    }
    property Timer delay: Timer {
        interval: 350
        onTriggered: {
            const next = JSON.parse(JSON.stringify(root.snapshot)), request = root.queued;
            if (root.queuedGeneration !== root.generation || !root.active) { next.busy = false; root.snapshot = next; return; }
            if (request.action === "brightness") {
                for (const light of next.backlights) if (light.id === request.device) { light.percent = request.percent; light.value = request.percent * 10; }
                next.message = "Sample brightness updated";
            } else if (request.action === "profile") { next.profiles.active = request.profile; next.message = "Sample profile updated"; }
            else if (request.action === "prepare") {
                next.prompt = {token:root.sequence++,kind:request.kind,session_id:next.session.id,expires_at:Date.now()+30000};
                next.message = "Review the session action";
            } else if (request.action === "confirm") {
                next.prompt = null; next.message = "Sample session request accepted; no system action performed";
            } else next.message = "Sample power state refreshed";
            next.busy = false;
            root.snapshot = next;
        }
    }
    property Timer expiry: Timer {
        interval: 500; repeat: true; running: !!root.snapshot.prompt
        onTriggered: if (root.snapshot.prompt && Date.now() >= root.snapshot.prompt.expires_at) root.snapshot = Object.assign({}, root.snapshot, {prompt:null,message:"Session confirmation expired"})
    }
    Component.onCompleted: {
        const mode = Quickshell.env("HYPRLUME_POWER_STATE"), next = JSON.parse(JSON.stringify(snapshot));
        if (mode === "unavailable") {
            next.battery = {available:false,present:false,percentage:null,state:"unknown",seconds_remaining:null,error:"UPower unavailable"};
            next.backlights = []; next.profiles = {available:false,active:null,choices:[],degraded:"",error:"Power profiles daemon is unavailable"};
            next.session = {available:false,id:"",suspend:"no",logout:"no",reboot:"no",power_off:"no",error:"No active graphical login is available"};
        } else if (mode === "desktop") { next.battery.present = false; next.battery.percentage = null; next.battery.seconds_remaining = null; }
        else if (mode === "inhibited") { next.session.suspend = "inhibited"; next.session.reboot = "inhibited"; next.session.power_off = "inhibited"; }
        else if (mode === "review") next.prompt = {token:sequence++,kind:"suspend",session_id:"7",expires_at:Date.now()+30000};
        snapshot = next;
    }
}
