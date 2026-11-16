import QtQuick
import Quickshell
import Quickshell.Io

// Samples CPU, memory, GPU, network, disk, and running containers from one
// long-lived shell script. One process and one sample per two seconds; no polling
// timer per metric and no IPC hop through the Rust daemon.
Item {
    id: root

    property bool active: true
    onActiveChanged: { restart.stop(); sampler.running = active; }

    // -- Latest sample -------------------------------------------------------
    property int cpu: 0
    property int mem: 0
    property int memMb: 0
    property int gpu: -1
    property int netUp: 0
    property int netDown: 0
    property real diskUsed: -1
    property real diskTotal: -1
    property string load: "0.00"
    property int uptimeMinutes: 0

    // -- Containers ----------------------------------------------------------
    property int containerCount: 0
    property string containerName: ""
    property real containerCpu: 0
    property string containerMem: ""
    property string containerRuntime: ""

    // -- History (oldest first) ---------------------------------------------
    readonly property int historyLength: 48
    property var cpuHistory: []
    property var netHistory: []

    readonly property string scriptPath: Quickshell.shellPath("services/system-metrics.sh")

    readonly property bool gpuAvailable: gpu >= 0

    function consume(line) {
        if (!line || line.length < 2)
            return;

        const parts = line.split(" ");
        const kind = parts[0];
        const field = {};
        for (let i = 1; i < parts.length; i++) {
            const split = parts[i].indexOf("=");
            if (split > 0)
                field[parts[i].slice(0, split)] = parts[i].slice(split + 1);
        }

        if (kind === "S") {
            root.cpu = number(field.cpu);
            root.mem = number(field.mem);
            root.memMb = number(field.memmb);
            root.gpu = number(field.gpu);
            root.netUp = number(field.netup);
            root.netDown = number(field.netdown);
            root.diskUsed = number(field.disk);
            root.diskTotal = number(field.disktotal);
            root.load = field.load ?? "0.00";
            root.uptimeMinutes = number(field.uptime);
            root.pushHistory();
        } else if (kind === "C") {
            root.containerCount = number(field.count);
            root.containerName = field.name ?? "";
            root.containerCpu = number(field.cpu);
            root.containerMem = field.mem ?? "";
            root.containerRuntime = field.runtime ?? "";
        }
    }

    function number(value) {
        const parsed = Number(value);
        return isNaN(parsed) ? 0 : parsed;
    }

    function pushHistory() {
        const cpuSamples = root.cpuHistory.concat([root.cpu / 100]);
        root.cpuHistory = cpuSamples.slice(-root.historyLength);

        const netSamples = root.netHistory.concat([(root.netUp + root.netDown) / 1024]);
        root.netHistory = netSamples.slice(-root.historyLength);
    }

    // -- Presentation helpers ------------------------------------------------
    function formatRate(kibPerSecond) {
        if (kibPerSecond >= 1024)
            return (kibPerSecond / 1024).toFixed(1) + " MB/s";
        return Math.round(kibPerSecond) + " KB/s";
    }

    function formatBytes(bytes) {
        if (bytes < 0)
            return "—";
        const units = ["B", "KB", "MB", "GB", "TB"];
        let value = bytes;
        let unit = 0;
        while (value >= 1024 && unit < units.length - 1) {
            value /= 1024;
            unit++;
        }
        return (unit === 0 ? Math.round(value) : value.toFixed(value >= 10 ? 0 : 1)) + " " + units[unit];
    }

    function formatUptime(minutes) {
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);
        if (days > 0)
            return days + "d " + (hours % 24) + "h";
        if (hours > 0)
            return hours + "h " + (minutes % 60) + "m";
        return minutes + "m";
    }

    Process {
        id: sampler
        command: ["sh", root.scriptPath]
        running: root.active

        onExited: { if (root.active) restart.restart(); }

        stdout: SplitParser {
            onRead: line => root.consume(line)
        }
    }

    Timer {
        id: restart
        interval: 2000
        repeat: false
        onTriggered: sampler.running = root.active
    }
}
