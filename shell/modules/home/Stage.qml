import QtQuick
import QtQuick.Layouts
import qs.components
import qs.models
import qs.theme

// A real command console: every output line is derived from daemon state.
// A terminal shortcut opens an allowlisted terminal through the same preview.
GlassSurface {
    id: root
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    property int mode: 0
    property int activeTab: 0
    onModeChanged: activeTab = mode === 4 || mode === 5 ? 2 : mode === 1 ? 1 : 0
    readonly property var snapshot: backend.snapshot
    CommandState {
        id: commandState
        snapshot: root.snapshot
    }
    readonly property var output: {
        const lines = [];
        for (const action of (snapshot.actions ?? [])) lines.push({text: action, kind: "plan"});
        for (const receipt of (snapshot.receipts ?? [])) lines.push({text: receipt, kind: "ok"});
        if (lines.length === 0) {
            lines.push({text: snapshot.message || "Your desktop, within reach.", kind: "status"});
            lines.push({text: snapshot.voice === "off" ? "Voice is off. Enable it when you need it." : "Voice is ready. Say Hey Pilot.", kind: "dim"});
            lines.push({text: backend.online ? "Connected to your local desktop." : "Waiting for the local agent…", kind: "dim"});
        }
        return lines.slice(0, 5);
    }
    radius: 25
    glowColor: Theme.cyan
    secondaryColor: Theme.violet

    function submit() {
        const text = input.text.trim();
        if (text && !commandState.busy && !commandState.review && root.backend.online) {
            root.backend.send({op: "command", text: text, dry_run: true});
            input.text = "";
        }
    }
    Row {
        x: 25; y: 13; spacing: 8
        Repeater {
            model: ["hyprlume / console", "Session activity", "System − live"]
            delegate: Rectangle {
                required property string modelData
                required property int index
                width: index === 0 ? 205 : 155; height: 38; radius: 11
                color: root.activeTab === index ? "#3921416a" : "#58051120"
                border.width: 1
                border.color: root.activeTab === index ? Theme.violet : "#3157b1d8"
                Row {
                    anchors.centerIn: parent; spacing: 8
                    Icon { name: index === 0 ? "folder" : index === 1 ? "cloud" : "chart"; color: Theme.cyan; size: 16 }
                    Text { text: modelData; color: root.activeTab === index ? Theme.ink : Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 12 }
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.activeTab = index }
            }
        }
        IconTile { iconName: "plus"; size: 36; iconSize: 16; onClicked: root.backend.send({op: "toggle"}) }
    }
    Rectangle { x: 14; y: 59; width: parent.width - 28; height: 1; color: "#395cd8ff" }
    Row {
        x: 29; y: 83; spacing: 10
        Text { text: "❯"; color: Theme.amber; font.family: Theme.fontMono; font.pixelSize: 21 }
        Chip {
            text: root.snapshot.active_app.length > 0 ? root.snapshot.active_app : "local session"
            textColor: Theme.ink
            fillColor: "#805528a3"; strokeColor: "#ba9a6eff"
            horizontalPadding: 18
        }
        Chip {
            text: "workspace " + (root.snapshot.workspace ?? "—")
            textColor: Theme.mint; fillColor: "#50206037"; strokeColor: "#7068f0b1"; mono: true
        }
    }
    Row {
        x: 30; y: 128; spacing: 14
        visible: root.activeTab !== 2
        Text { text: "{"; color: Theme.cyan; font.family: Theme.fontMono; font.pixelSize: 22 }
        TextInput {
            id: input
            objectName: "consoleInput"
            width: Math.max(120, root.width - 250); height: 30
            color: Theme.ink; font.family: Theme.fontMono; font.pixelSize: 21
            clip: true; selectByMouse: true; maximumLength: 512
            enabled: root.backend.online && !commandState.busy && !commandState.review
            onAccepted: root.submit()
            Text {
                anchors.fill: parent
                visible: input.text.length === 0
                text: root.snapshot.transcript || "What would you like to do?"
                color: root.snapshot.transcript ? Theme.ink : Theme.inkFaint
                font: input.font
                elide: Text.ElideRight
            }
            Accessible.name: "Desktop command. Enter previews the action."
        }
    }
    IconTile {
        anchors.right: parent.right; anchors.rightMargin: 22; y: 78
        size: 30; iconName: "expand"; iconSize: 15
        onClicked: root.backend.send({op: "command", text: "launch terminal", dry_run: true})
        Accessible.name: "Open terminal preview"
    }
    Column {
        x: 34; y: 181; width: Math.max(140, parent.width - 245); spacing: 9
        visible: root.activeTab !== 2
        Repeater {
            model: root.output
            delegate: Row {
                required property var modelData
                width: parent.width; spacing: 13
                Text {
                    text: modelData.kind === "ok" ? "✓" : modelData.kind === "plan" ? "◇" : "│"
                    color: modelData.kind === "ok" ? Theme.mint : Theme.cyan
                    font.family: Theme.fontMono; font.pixelSize: 14
                }
                Text {
                    width: parent.width - 25
                    text: modelData.text
                    color: modelData.kind === "ok" ? "#ccf4c1" : modelData.kind === "dim" ? Theme.inkFaint : Theme.inkDim
                    font.family: Theme.fontMono; font.pixelSize: 13
                    elide: Text.ElideRight
                }
            }
        }
    }
    Column {
        x: 40; y: 140; spacing: 16; visible: root.activeTab === 2
        Text { text: root.mode === 4 ? "LOCAL CONTAINERS / LIVE" : root.mode === 5 ? "NETWORK / LIVE" : "YOUR SYSTEM / LIVE"; color: Theme.cyan; font.family: Theme.fontMono; font.pixelSize: 12; font.letterSpacing: 2 }
        Text { text: root.mode === 4 ? (root.metrics.containerRuntime || "No runtime") + " · " + root.metrics.containerCount + " running" : "CPU    " + root.metrics.cpu + "%   ·   RAM    " + root.metrics.mem + "%"; color: Theme.ink; font.family: Theme.fontMono; font.pixelSize: 20 }
        Text { text: root.mode === 4 ? (root.metrics.containerName || "No active containers") : "↑ " + root.metrics.formatRate(root.metrics.netUp) + "    ↓ " + root.metrics.formatRate(root.metrics.netDown); color: Theme.inkDim; font.family: Theme.fontMono; font.pixelSize: 15 }
        Text { text: "Uptime  " + root.metrics.formatUptime(root.metrics.uptimeMinutes); color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 13 }
    }
    StageBackdrop { x: 7; anchors.bottom: parent.bottom; anchors.bottomMargin: 5; width: parent.width - 14; height: 193 }
    ClockCard { anchors.right: parent.right; anchors.rightMargin: 22; y: 180; clock: root.clock; metrics: root.metrics }
    Row {
        anchors.bottom: parent.bottom; anchors.bottomMargin: 11; x: 25; spacing: 7
        StatusDot { anchors.verticalCenter: parent.verticalCenter; size: 5; color: root.backend.online ? Theme.mint : Theme.rose }
        Text { text: root.backend.online ? "LOCAL AGENT CONNECTED" : "AGENT OFFLINE"; color: "#91acc8"; font.family: Theme.fontMono; font.pixelSize: 8; font.letterSpacing: 1.1 }
    }
}
