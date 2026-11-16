import QtQuick
import Quickshell
import qs.services

ShellRoot {
    property bool sent: false
    Backend {
        id: backend
        onSnapshotChanged: {
            if (snapshot.phase === "ready" && !sent) {
                sent = true
                send({op: "command", text: "workspace 2", dry_run: true})
            }
            if (snapshot.phase === "review" && snapshot.confirmation !== null && snapshot.actions.length === 1) {
                console.log("HYPRLUME_IPC_PASS")
                Qt.quit()
            }
        }
    }
    Timer { interval: 5000; running: true; onTriggered: { console.error("IPC smoke timed out"); Qt.exit(1) } }
}
