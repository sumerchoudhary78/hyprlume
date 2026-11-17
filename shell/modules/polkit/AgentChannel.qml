import QtQuick
import Quickshell
import Quickshell.Io

Scope {
    id: root
    required property list<string> command
    required property bool available
    property var request: null
    property var prompt: null
    property bool choosing: false
    property string notice: ""
    property bool noticeIsError: false
    signal registered()
    signal workerExited(int status)

    function send(packet) {
        worker.write(JSON.stringify(packet) + "\n");
    }

    function clear() {
        prompt = null;
        request = null;
        choosing = false;
        notice = "";
        noticeIsError = false;
    }

    function start(account) {
        if (!request || !available || !choosing) return;
        choosing = false;
        notice = "";
        send({kind: "start", "id": request.id, account: account});
    }

    function respond(value) {
        if (!request || !prompt || !available) return;
        const packet = {kind: "respond", "id": request.id, prompt: prompt.prompt, response: value};
        prompt = null;
        notice = "";
        send(packet);
    }

    function cancel() {
        if (request) send({kind: "cancel", "id": request.id});
        clear();
    }

    function receive(packet) {
        if (packet.kind === "registered") {
            registered();
        } else if (packet.kind === "open") {
            if (!available) {
                send({kind: "cancel", "id": packet.id});
                return;
            }
            clear();
            request = packet;
            choosing = true;
            if (packet.accounts.length === 1) start(0);
        } else if (request && packet.id === request.id) {
            switch (packet.kind) {
            case "prompt":
                prompt = packet;
                break;
            case "notice":
                notice = packet.message;
                noticeIsError = packet.error;
                break;
            case "retry":
                prompt = null;
                choosing = true;
                notice = "Authentication failed. Try again or cancel.";
                noticeIsError = true;
                break;
            case "close":
                clear();
                break;
            }
        }
    }

    onAvailableChanged: {
        send({kind: "available", value: available});
        if (!available) clear();
    }

    Process {
        id: worker
        command: root.command
        stdinEnabled: true
        running: true
        onStarted: root.send({kind: "available", value: root.available})
        onExited: (code, status) => {
            root.clear();
            root.workerExited(status === 0 ? code : 1);
        }
        stdout: SplitParser {
            onRead: line => {
                try {
                    root.receive(JSON.parse(line));
                } catch (_) {
                    root.cancel();
                    worker.running = false;
                }
            }
        }
        stderr: SplitParser {
            onRead: line => console.warn(line)
        }
    }
}
