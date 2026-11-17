import QtQuick
import Quickshell
import qs.fixtures
import qs.modules.polkit

ShellRoot {
    id: root
    LockGate { id: login }
    AgentChannel {
        id: channel
        command: ["python3", Quickshell.env("HYPRLUME_POLKIT_TEST_SCRIPT"), "--worker-fixture"]
        available: login.available
    }
    FloatingWindow {
        title: "Hyprlume Polkit Preview"
        visible: true
        implicitWidth: 840
        implicitHeight: 560
        Item {
            id: content
            anchors.fill: parent
            Loader {
                anchors.fill: parent
                active: channel.request !== null && login.available
                sourceComponent: AuthDialog { agent: channel }
            }
            ControlProbe { targetItem: content; backend: null }
        }
    }
}
