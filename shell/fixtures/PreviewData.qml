import QtQuick
import Quickshell
import qs.services

// Explicitly synthetic data for design previews and interaction tests only.
// The normal shell entry never instantiates this fixture.
QtObject {
    id: fixture
    property QtObject backend: QtObject {
        id: sampleBackend
        property bool online: true
        property string error: ""
        signal response(int id, bool ok, var result, string error)
        property var settings: ({revision:1,values:{preferences:{appearance:{reduced_effects:false,accent:"cyan",scale:1,refraction:16,metrics:true,high_contrast:false,app_color:"dark"},shortcuts:{launcher:"SUPER+SPACE",windows:"SUPER+TAB",tools:"SUPER+V",lock:"SUPER+L"},services:{clipboard_history:false,notifications_dnd:false},input:{layouts:"us",variants:"",repeat_rate:25,repeat_delay:600,pointer_speed:0,left_handed:false,mouse_natural_scroll:false,touchpad_natural_scroll:false,tap_to_click:true,disable_while_typing:true,shortcut_bindings:false,system_key_bindings:false},idle:{enabled:false,dim_after_seconds:180,lock_after_seconds:300,screen_off_after_seconds:360,suspend_after_seconds:0,dim_percent:20},desktop:{wallpaper:"",gtk_theme:"",qt_style:"system",locale:"",timezone:"",clock_24h:true,qt_scale:1,accessibility_bus:false}},voice:{asr_dir:"/preview/models/asr",kws_dir:"/preview/models/kws",tts_dir:"/preview/models/tts",keywords_file:"/preview/models/kws/keywords.txt",threads:2,execute_commands:false,speak_results:true},llm:{endpoint:null,model:"local",timeout_seconds:20}},path:"/preview/config.toml",restart_required:false,message:"Preview settings are never written to disk",hyprlang:"bind = SUPER, SPACE, exec, hyprlume launcher",lua:"hl.bind(\"SUPER + SPACE\", hl.dsp.exec_cmd(\"hyprlume launcher\"))"})
        property var lastRequest: ({})
        property int requestCount: 0
        property int pairCount: 0
        property QtObject audio: AudioFixture { active: sampleBackend.snapshot.desktop_panel === "audio" }
        property QtObject launcher: LauncherFixture {}
        property QtObject tray: TrayFixture { active: sampleBackend.snapshot.desktop_panel === "tray" }
        property QtObject powerService: PowerFixture { active: sampleBackend.snapshot.desktop_panel === "power" }
        property var idle: ({enabled:false,available:false,presentation:false,recording_inhibits:false,dimmed:false,display_off:false,phase:"disabled",message:"Automatic idle actions are off",error:""})
    property var locking: ({available:true,active:false,phase:"idle",message:"Synthetic lock control · no session will be locked"})
        property var clipboard: ({available:true,enabled:false,error:"",items:[{id:1,preview:"cargo build --release",bytes:21},{id:2,preview:"The next idea is already within reach.",bytes:36}]})
        property var capture: ({screenshot_available:true,recording_available:true,region_available:true,busy:false,recording:false,output:"",message:"Screen capture is idle · recordings have no audio",error:""})
        property var power: powerService.snapshot
        property var notifications: ({
            available:true,error:"",panel_open:Quickshell.env("HYPRLUME_NOTIFICATIONS_PREVIEW") === "1",dnd:false,unread:3,
            items:[
                {id:1,revision:1,app_name:"Build system",summary:"Your release is ready",body:"All checks passed. The optimized build is ready for review.",actions:[{key:"default",label:"Open build"}],urgency:1,created_at:0,active:true,popup:Quickshell.env("HYPRLUME_NOTIFICATION_POPUPS") === "1",read:false},
                {id:2,revision:2,app_name:"Calendar",summary:"Design review in 10 minutes",body:"Observatory workspace · Desktop services",actions:[{key:"join",label:"Join"},{key:"later",label:"Remind later"}],urgency:1,created_at:0,active:true,popup:Quickshell.env("HYPRLUME_NOTIFICATION_POPUPS") === "1",read:false},
                {id:3,revision:3,app_name:"System",summary:"Battery needs attention",body:"Sample critical alert. Critical notifications wait for dismissal.",actions:[],urgency:2,created_at:0,active:true,popup:Quickshell.env("HYPRLUME_NOTIFICATION_POPUPS") === "1",read:false}
            ]
        })
        property var connectivity: ({
            panel_open:Quickshell.env("HYPRLUME_CONNECTIVITY_PREVIEW") === "1",busy:false,message:"Your wireless connections",error:"",prompt:null,
            wifi:{available:true,enabled:true,hardware_enabled:true,error:"",interfaces:[{name:"wlan0",connected:true,ssid:"Orbital uplink"}],networks:[
                {id:"wifi-home",interface:"wlan0",ssid:"Orbital uplink",bssid:"00:11:22:33:44:55",strength:94,security:"wpa3",connected:true,saved_uuid:"sample-home",frequency:5180},
                {id:"wifi-lab",interface:"wlan0",ssid:"Nebula lab",bssid:"00:11:22:33:44:66",strength:78,security:"wpa2",connected:false,saved_uuid:null,frequency:5745},
                {id:"wifi-guest",interface:"wlan0",ssid:"Observatory guest",bssid:"00:11:22:33:44:77",strength:61,security:"open",connected:false,saved_uuid:null,frequency:2412}
            ]},
            bluetooth:{available:true,error:"",adapters:[{id:"hci0",name:"Orbital workstation",powered:true,discovering:false}],devices:[
                {adapter:"hci0",address:"AA:BB:CC:00:00:01",name:"Studio headphones",icon:"audio-headset",paired:true,connected:true,trusted:true},
                {adapter:"hci0",address:"AA:BB:CC:00:00:02",name:"Pilot keyboard",icon:"input-keyboard",paired:false,connected:false,trusted:false}
            ]}
        })
        property Timer connectionDelay: Timer {
            interval: 450
            onTriggered: sampleBackend.connectivity = Object.assign({}, sampleBackend.connectivity, {busy:false,message:"Sample connection updated"})
        }
        property Timer pairingRefresh: Timer {
            interval: 200; repeat: true
            running: Quickshell.env("HYPRLUME_QA") === "1" && sampleBackend.connectivity.prompt !== null
            // Simulate unrelated D-Bus snapshots while the user types a PIN.
            onTriggered: sampleBackend.connectivity = JSON.parse(JSON.stringify(sampleBackend.connectivity))
        }
        property var snapshot: ({phase:"ready",message:"Your desktop, within reach.",transcript:"workspace two",actions:[],receipts:["Focused Development workspace","Desktop state verified","Ready for your next command"],confirmation:null,panel_open:false,desktop_panel:Quickshell.env("HYPRLUME_SETTINGS_PREVIEW") === "1" ? "settings" : Quickshell.env("HYPRLUME_TOOLS_PREVIEW") === "1" ? (Quickshell.env("HYPRLUME_CAPTURE_PREVIEW") === "1" ? "capture" : "clipboard") : Quickshell.env("HYPRLUME_LAUNCHER_PREVIEW") === "1" ? "launcher" : Quickshell.env("HYPRLUME_TRAY_PREVIEW") === "1" ? "tray" : Quickshell.env("HYPRLUME_POWER_PREVIEW") === "1" ? "power" : Quickshell.env("HYPRLUME_AUDIO_PREVIEW") === "1" ? "audio" : null,voice:"off",workspace:2,active_app:"hyprlume"})
        property Timer planningDelay: Timer {
            interval: 600
            onTriggered: sampleBackend.snapshot = Object.assign({}, sampleBackend.snapshot,
                sampleBackend.lastRequest.dry_run === false
                ? {phase:"ready",actions:[],receipts:["Sample direct action completed"],confirmation:null}
                : {phase:"review",actions:["Preview: " + sampleBackend.lastRequest.text],receipts:[],confirmation:42})
        }
        function send(operation) {
            const publicOperation = JSON.parse(JSON.stringify(operation));
            if (publicOperation.request && publicOperation.request.password) publicOperation.request.password = "[credential]";
            if (publicOperation.request && publicOperation.request.value) publicOperation.request.value = "[credential]";
            lastRequest = publicOperation; requestCount++;
            if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"request",operation:publicOperation}));
            if (operation.op === "display") {
                const sequence = requestCount;
                const request = operation.request;
                const displays = Object.assign({revision:1, outputs:[
                    {name:"eDP-1", label:"Notebook display", modes:["1920x1080@144.000Hz", "1920x1080@60.000Hz"],
                     active:{name:"eDP-1", mode:"1920x1080@144.000Hz", x:0, y:0, scale:1}},
                    {name:"DP-1", label:"Dock display", modes:["2560x1440@60.000Hz"],
                     active:{name:"DP-1", mode:"2560x1440@60.000Hz", x:1920, y:0, scale:1}}
                ], busy:false, preview:null, message:"Sample display layout."}, snapshot.displays);
                if (request.action === "preview") {
                    displays.busy = true;
                    displays.preview = {token: 33, expires_at: Date.now() + 15000};
                    displays.changes = request.changes;
                } else if (request.action === "keep" || request.action === "revert") {
                    if (request.action === "keep") displays.outputs = displays.outputs.map(
                        output => Object.assign({}, output, {active:displays.changes.find(setting => setting.name === output.name)}));
                    displays.preview = null;
                    displays.busy = false;
                    displays.revision++;
                }
                snapshot = Object.assign({}, snapshot, {displays:displays});
                Qt.callLater(() => sampleBackend.response(sequence, true, {accepted:true}, ""));
                return sequence;
            }
            if (operation.op === "desktop_preferences") {
                const sequence = requestCount;
                const request = operation.request;
                const result = request.mime ? {
                    mime: request.mime, desktop_id: request.desktop_id || "preview-browser.desktop",
                    app_name: "Sample application", choices: [{id: "preview-browser.desktop", name: "Sample browser"}],
                    message: "Sample app association."
                } : request.action === "write_environment" ? {
                    profile: "/preview/environment.d/80-hyprlume-desktop.conf",
                    message: "Sample app profile."
                } : {message: "Sample screen reader action."};
                Qt.callLater(() => sampleBackend.response(sequence, true, result, ""));
                return sequence;
            }
            if (operation.op === "storage") {
                const sequence = requestCount;
                const request = operation.request;
                const storage = JSON.parse(JSON.stringify(snapshot.storage));
                const volume = storage.drives[0]?.volumes.find(entry => entry.id === request.id);
                if (request.action === "unlock" && volume) { volume.locked = false; volume.filesystem = true; }
                if (request.action === "mount" && volume) volume.mounted = true;
                if (request.action === "unmount" && volume) volume.mounted = false;
                if (request.action === "eject") storage.drives = [];
                storage.message = "Sample drive action completed.";
                snapshot = Object.assign({}, snapshot, {storage: storage});
                Qt.callLater(() => sampleBackend.response(sequence, true, {accepted: true}, ""));
                return sequence;
            }
            if (operation.op === "credentials") {
                const sequence = requestCount;
                const action = operation.request.action;
                const credentials = Object.assign({provider: "running", login_locked: true, default_locked: true, busy: false, message: "Sample credential state."}, snapshot.credentials);
                if (action === "lock" || action === "unlock") {
                    credentials.login_locked = action === "lock";
                    credentials.default_locked = credentials.login_locked;
                }
                snapshot = Object.assign({}, snapshot, {credentials: credentials});
                Qt.callLater(() => sampleBackend.response(sequence, true, {accepted: true}, ""));
                return sequence;
            }
            if (operation.op === "input") {
                const sequence = requestCount;
                const result = {
                    keyboards: [{name: "preview-keyboard", active_keymap: "English (US)", main: true}],
                    profile: operation.request.action === "write_profile" ? "/preview/hyprlume-input.lua" : "",
                    message: "Preview input action."
                };
                Qt.callLater(() => sampleBackend.response(sequence, true, result, ""));
                return sequence;
            }
            if (operation.op === "portals_status") {
                const sequence = requestCount;
                const status = {
                    desktop: "Hyprlume:Hyprland",
                    profile_requested: true,
                    frontend: "running", hyprland: "running", gtk: "on_demand", appearance: "running",
                    interfaces: [
                        {label:"File dialogs", version:4}, {label:"Screen sharing", version:6},
                        {label:"Screenshots", version:2}, {label:"App shortcuts", version:1},
                        {label:"Appearance", version:2}
                    ],
                    message: "Sample portal state in the design preview."
                };
                Qt.callLater(() => sampleBackend.response(sequence, true, status, ""));
                return sequence;
            }
            if (operation.op === "settings") {
                const sequence=requestCount;
                if(operation.request.action==="save") settings=Object.assign({},settings,{revision:settings.revision+1,values:JSON.parse(JSON.stringify(operation.request.values)),message:"Preview settings saved in memory"});
                const result=settings;
                Qt.callLater(()=>sampleBackend.response(sequence,true,result,""));
                return sequence;
            }
            else if (operation.op === "clipboard") {
                const request=operation.request;
                if (request.action==="enable") clipboard=Object.assign({},clipboard,{enabled:request.enabled});
                if (request.action==="clear") clipboard=Object.assign({},clipboard,{items:[]});
                if (request.action==="remove") clipboard=Object.assign({},clipboard,{items:clipboard.items.filter(entry=>entry.id!==request.id)});
            }
            else if (operation.op === "capture") {
                const request=operation.request;
                capture=Object.assign({},capture,{busy:request.action==="record",recording:request.action==="record",message:"Synthetic capture only",output:"/preview/capture.png"});
                if (request.action!=="stop") snapshot=Object.assign({},snapshot,{desktop_panel:null});
            }
            else if (operation.op === "idle") idle=Object.assign({},idle,{presentation:operation.request.enabled});
            else if (operation.op === "power") powerService.send(operation.request);
            else if (operation.op === "panel") {
                if (operation.open) {
                    notifications = Object.assign({}, notifications, {panel_open:false});
                    connectivity = Object.assign({}, connectivity, {panel_open:false});
                    snapshot = Object.assign({}, snapshot, {panel_open:false,desktop_panel:operation.panel});
                    if (operation.panel === "storage" && !snapshot.storage) {
                        snapshot = Object.assign({}, snapshot, {storage: {
                            available: true, busy: false, message: "Sample drives",
                            drives: [{id: "fixture-drive", label: "Preview USB", can_eject: true, volumes: [
                                {id: "fixture-volume", label: "Documents", filesystem: true, mounted: true, encrypted: false, locked: false, protected: false},
                                {id: "fixture-encrypted", label: "Vault", filesystem: false, mounted: false, encrypted: true, locked: true, protected: false}
                            ]}]
                        }});
                    }
                } else if (snapshot.desktop_panel === operation.panel) snapshot = Object.assign({}, snapshot, {desktop_panel:null});
            }
            else if (operation.op === "notifications") {
                const action = operation.request;
                const next = JSON.parse(JSON.stringify(notifications));
                if (action.action === "toggle") {
                    next.panel_open = !next.panel_open;
                    if (next.panel_open) {
                        next.unread = 0;
                        for (const item of next.items) { item.read = true; item.popup = false; }
                        snapshot = Object.assign({}, snapshot, {panel_open:false,desktop_panel:null});
                        connectivity = Object.assign({}, connectivity, {panel_open:false});
                    }
                } else if (action.action === "close") next.panel_open = false;
                else if (action.action === "dnd") { next.dnd = action.enabled; if (next.dnd) for (const item of next.items) item.popup = false; }
                else if (action.action === "clear") { next.items = []; next.unread = 0; }
                else {
                    const item = next.items.find(item => item.id === action.id && item.revision === action.revision);
                    if (!item) return;
                    if (action.action === "remove") next.items = next.items.filter(item => item.id !== action.id);
                    else { item.active = false; item.popup = false; item.read = true; }
                    next.unread = next.items.filter(item => !item.read).length;
                }
                notifications = next;
            }
            else if (operation.op === "connectivity") {
                const action = operation.request;
                const next = JSON.parse(JSON.stringify(connectivity));
                next.error = "";
                if (action.action === "toggle") { next.panel_open = !next.panel_open; snapshot = Object.assign({}, snapshot, {panel_open:false,desktop_panel:null}); notifications = Object.assign({}, notifications, {panel_open:false}); }
                else if (action.action === "close") { next.panel_open = false; next.prompt = null; next.busy = false; }
                else if (action.action === "wifi_power") next.wifi.enabled = action.enabled;
                else if (action.action === "wifi_connect") { for (const network of next.wifi.networks) network.connected = network.id === action.network; }
                else if (action.action === "wifi_disconnect") { for (const network of next.wifi.networks) if (network.interface === action.interface) network.connected = false; }
                else if (action.action === "wifi_forget") { for (const network of next.wifi.networks) if (network.saved_uuid === action.uuid) network.saved_uuid = null; }
                else if (action.action === "bluetooth_power") next.bluetooth.adapters[0].powered = action.enabled;
                else if (action.action === "bluetooth_pair") {
                    const token = 77 + pairCount++;
                    next.busy = true;
                    next.prompt = {token:token,adapter:action.adapter,address:action.address,kind:pairCount === 1 ? "confirm" : "pin",code:pairCount === 1 ? "042619" : ""};
                }
                else if (action.action === "pair_reply") {
                    if (!next.prompt || next.prompt.token !== action.token) return;
                    if (action.accept && next.prompt.kind === "pin" && action.value !== "123456") return;
                    if (action.accept) for (const device of next.bluetooth.devices) if (next.prompt && device.address === next.prompt.address) { device.paired = true; device.connected = true; }
                    next.prompt = null; next.busy = false;
                } else if (action.action === "bluetooth_connect" || action.action === "bluetooth_disconnect") {
                    for (const device of next.bluetooth.devices) if (device.address === action.address) device.connected = action.action === "bluetooth_connect";
                } else if (action.action === "bluetooth_forget") {
                    for (const device of next.bluetooth.devices) if (device.address === action.address) { device.paired = false; device.connected = false; }
                }
                if (["wifi_scan","wifi_connect","wifi_disconnect","bluetooth_scan","bluetooth_connect","bluetooth_disconnect"].indexOf(action.action) >= 0) { next.busy = true; connectionDelay.restart(); }
                connectivity = next;
            }
            else if (operation.op === "toggle") {
                notifications = Object.assign({}, notifications, {panel_open:false});
                connectivity = Object.assign({}, connectivity, {panel_open:false});
                snapshot = Object.assign({}, snapshot, {panel_open:!snapshot.panel_open || !!snapshot.desktop_panel,desktop_panel:null});
            }
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
        property int nextCount: 0
        function label(w) { return w.name; }
        function countFor(w) { return w.id === 2 ? 0 : w.id === 3 ? 2 : 1; }
        function sameWorkspace(a, b) { return !!a && !!b && a.name === b.name; }
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
    property QtObject state: ShellState { backend: sampleBackend }
    Component.onCompleted: {
        const menu = Quickshell.env("HYPRLUME_PREVIEW_MENU");
        if (menu === "overview" || menu === "controls") state.toggleMenu(menu);
        if (Quickshell.env("HYPRLUME_COMMAND_REVIEW") === "1") {
            backend.snapshot = Object.assign({}, backend.snapshot, {
                phase: "review", message: "Review the action before running it",
                transcript: "volume 35", actions: ["Set output volume to 35%"], receipts: [],
                confirmation: 42, panel_open: true
            });
        }
        const mode = Quickshell.env("HYPRLUME_CONNECTIVITY_STATE");
        if (mode === "unavailable") {
            const next = JSON.parse(JSON.stringify(backend.connectivity));
            next.wifi = {available:false,enabled:false,hardware_enabled:false,error:"No Wi-Fi adapter found",interfaces:[],networks:[]};
            next.bluetooth = {available:false,error:"No Bluetooth adapter found",adapters:[],devices:[]};
            backend.connectivity = next;
        } else if (mode === "pairing" || mode === "pin") {
            backend.connectivity = Object.assign({}, backend.connectivity, {busy:true,prompt:{token:77,adapter:"hci0",address:"AA:BB:CC:00:00:02",kind:mode === "pin" ? "pin" : "confirm",code:mode === "pin" ? "" : "042619"}});
        }
    }
}
