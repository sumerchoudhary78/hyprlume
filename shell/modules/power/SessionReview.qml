import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

Item {
    id: root
    required property var backend
    readonly property var prompt: backend.power.prompt
    readonly property string label: prompt ? ({suspend:"Suspend",logout:"Log out",reboot:"Restart",power_off:"Shut down"}[prompt.kind] || "Continue") : ""
    property int remaining: 0
    function tick() { remaining = prompt ? Math.max(0, Math.ceil((prompt.expires_at - Date.now()) / 1000)) : 0; }
    function cancel() { if (prompt) backend.send({op:"power",request:{action:"cancel",token:prompt.token}}); }
    focus: true
    Keys.onEscapePressed: root.cancel()
    Component.onCompleted: { tick(); forceActiveFocus(); }
    Rectangle { anchors.fill: parent; radius: Theme.radiusPanel; color: "#c9050c1c"; MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons } }
    GlassSurface {
        anchors.centerIn: parent; width: parent.width - 32
        height: Math.min(parent.height - 32, content.implicitHeight + 40)
        radius: 22
        ColumnLayout {
            id: content
            anchors.fill: parent; anchors.margins: 20; spacing: 15
            Icon { name: "power"; size: 27; color: Theme.amber }
            Text { Layout.fillWidth: true; text: root.label + "?"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 24; font.weight: Font.DemiBold }
            Text {
                Layout.fillWidth: true; wrapMode: Text.Wrap
                text: !root.prompt ? "" : root.prompt.kind === "suspend" ? "Lock the screen, then put the computer to sleep? Sleep is cancelled if the compositor cannot confirm the lock."
                    : root.prompt.kind === "logout" ? "End graphical session " + root.prompt.session_id + "? Save your work before continuing."
                    : "Save your work first. This will " + (root.prompt.kind === "reboot" ? "restart" : "shut down") + " the computer and affect all sessions."
                color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 12
            }
            Text { text: "Confirmation expires in " + root.remaining + "s"; color: Theme.amber; font.family: Theme.fontMono; font.pixelSize: 10 }
            Text { visible: !!root.prompt && root.backend.power.session[root.prompt.kind] === "challenge"; Layout.fillWidth: true; wrapMode: Text.Wrap; text: "System authorization may be requested."; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11 }
            RowLayout {
                Layout.fillWidth: true; spacing: 10
                GlassButton { objectName: "cancelPowerAction"; text: "Keep working"; Layout.fillWidth: true; onClicked: root.cancel() }
                GlassButton { objectName: "confirmPowerAction"; text: root.label; primary: true; Layout.fillWidth: true; enabled: !!root.prompt && root.remaining > 0 && !root.backend.power.busy; onClicked: root.backend.send({op:"power",request:{action:"confirm",token:root.prompt.token}}) }
            }
        }
    }
    Timer { interval: 500; repeat: true; running: !!root.prompt; onTriggered: root.tick() }
}
