import QtQuick
import Quickshell

// Explicitly synthetic data for design previews and interaction tests only.
// The normal shell entry never instantiates this fixture.
QtObject {
    id: fixture
    property QtObject backend: QtObject {
        id: sampleBackend
        property bool online: true
        property string error: ""
        property var lastRequest: ({})
        property int requestCount: 0
        property var snapshot: ({phase:"ready",message:"Your desktop, within reach.",transcript:"workspace two",actions:[],receipts:["Focused Development workspace","Desktop state verified","Ready for your next command"],confirmation:null,panel_open:false,voice:"off",workspace:2,active_app:"hyprlume"})
        property Timer planningDelay: Timer {
            interval: 600
            onTriggered: sampleBackend.snapshot = Object.assign({}, sampleBackend.snapshot,
                sampleBackend.lastRequest.dry_run === false
                ? {phase:"ready",actions:[],receipts:["Sample direct action completed"],confirmation:null}
                : {phase:"review",actions:["Preview: " + sampleBackend.lastRequest.text],receipts:[],confirmation:42})
        }
        function send(operation) {
            lastRequest = operation; requestCount++;
            if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"request",operation:operation}));
            if (operation.op === "toggle") snapshot = Object.assign({}, snapshot, {panel_open:!snapshot.panel_open});
            else if (operation.op === "voice") snapshot = Object.assign({}, snapshot, {voice:operation.mode});
            else if (operation.op === "command") {
                snapshot = Object.assign({}, snapshot, {phase:"planning",transcript:operation.text,actions:[],receipts:[],confirmation:null,panel_open:true});
                planningDelay.restart();
            }
            else if (operation.op === "confirm") snapshot = Object.assign({}, snapshot, {actions:[],receipts:["Sample action confirmed"],confirmation:null});
            else if (operation.op === "cancel") {
                planningDelay.stop();
                snapshot = Object.assign({}, snapshot, {phase:"ready",actions:[],confirmation:null});
            }
        }
    }
    property QtObject workspaces: QtObject {
        property var all: [{id:2,name:"Development"},{id:3,name:"Research"},{id:4,name:"Operations"},{id:5,name:"Personal"}]
        property var focused: all[0]
        property bool desktopVisible: true
        property int focusedWindowCount: 0
        property int nextCount: 0
        function label(w) { return w.name; }
        function countFor(w) { return w.id === 2 ? 3 : w.id === 3 ? 2 : 1; }
        function isFocused(w) { return focused && w.id === focused.id; }
        function focus(w) { focused = w; if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"workspace",name:w.name})); }
        function focusNext() { nextCount++; focused = all[(all.indexOf(focused)+1)%all.length]; }
        function focusPrevious() { focused = all[(all.indexOf(focused)+all.length-1)%all.length]; }
    }
    property QtObject metrics: QtObject {
        property int cpu: 18
        property int mem: 62
        property int memMb: 8240
        property int gpu: 35
        property bool gpuAvailable: true
        property int netUp: 185
        property int netDown: 1240
        property real diskUsed: 81202040832
        property real diskTotal: 250000000000
        property string load: "0.42"
        property int uptimeMinutes: 247
        property int containerCount: 2
        property string containerName: "dev-workspace"
        property real containerCpu: 2.8
        property string containerMem: "384 MiB"
        property string containerRuntime: "podman"
        property var cpuHistory: [0.10,0.12,0.38,0.34,0.4,0.49,0.43,0.38,0.50,0.46,0.62,0.43,0.41,0.48,0.43,0.39,0.44,0.33,0.35,0.46,0.51,0.48,0.57,0.41,0.39,0.44,0.31,0.29,0.35,0.39,0.45,0.35,0.49,0.39,0.52,0.57,0.48,0.64,0.60,0.54,0.41,0.47,0.51,0.43,0.40,0.45,0.49,0.46]
        property var netHistory: cpuHistory
        function formatRate(n) { return n >= 1024 ? (n/1024).toFixed(1)+" MB/s" : n+" KB/s"; }
        function formatBytes(n) { return (n/1073741824).toFixed(0)+" GB"; }
        function formatUptime(n) { return Math.floor(n/60)+"h "+n%60+"m"; }
    }
    property QtObject clock: QtObject {
        readonly property string time: "15:27"
        readonly property string date: "Mon, 16 Nov"
    }
    property QtObject state: QtObject {
        property int mode: 0
        onModeChanged: if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"mode",mode:mode}));
        property bool reducedEffects: Quickshell.env("HYPRLUME_REDUCED_EFFECTS") === "1"
    }
}
