import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var backend
    readonly property var status: backend.snapshot.credentials || {}
    property string error: ""
    property int pending: 0
    spacing: 14

    function send(action, collection) {
        const request = {action: action};
        if (collection) request.collection = collection;
        error = "";
        pending = backend.send({op: "credentials", request: request}) || 0;
        if (!pending) error = backend.error;
    }

    Component.onCompleted: send("refresh")
    Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (id !== root.pending) return;
            root.pending = 0;
            if (!ok) root.error = error;
        }
        function onOnlineChanged() {
            if (root.backend.online) root.send("refresh");
            else root.pending = 0;
        }
    }
    RowLayout {
        Layout.fillWidth: true
        Icon { name: "lock"; size: 20; color: Theme.cyan }
        Text {
            Layout.fillWidth: true
            text: "Credential storage"
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: 18
        }
        GlassButton {
            objectName: "credentialRefresh"
            text: "Refresh"
            compact: true
            enabled: root.backend.online && !root.pending && !root.status.busy
            onClicked: root.send("refresh")
        }
        GlassButton {
            objectName: "credentialActivate"
            text: "Start provider"
            visible: root.status.provider === "on_demand"
            enabled: !root.pending && !root.status.busy
            onClicked: root.send("activate")
        }
    }
    Text {
        Layout.fillWidth: true
        text: root.error || root.status.error || (root.status.busy ? "Waiting for the credential provider…" : root.status.message) || "Checking the session keyring…"
        color: root.error || root.status.error ? Theme.amber : Theme.inkDim
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
        font.pixelSize: 12
    }
    Repeater {
        model: [{id: "login", label: "Login collection"}, {id: "default", label: "Default collection"}]
        RowLayout {
            id: collection
            required property var modelData
            readonly property var locked: root.status[modelData.id + "_locked"]
            Layout.fillWidth: true
            visible: root.status.provider === "running"
            ColumnLayout {
                Layout.fillWidth: true
                Text { text: collection.modelData.label; color: Theme.ink; font.pixelSize: 14 }
                Text {
                    text: collection.locked === true ? "Locked" : collection.locked === false ? "Unlocked" : "No collection assigned"
                    color: collection.locked === false ? Theme.mint : Theme.inkFaint
                    font.pixelSize: 12
                }
            }
            GlassButton {
                objectName: "credential_" + collection.modelData.id
                text: collection.locked ? "Unlock…" : "Lock"
                enabled: root.backend.online && !root.pending && !root.status.busy && typeof collection.locked === "boolean"
                onClicked: root.send(collection.locked ? "unlock" : "lock", collection.modelData.id)
            }
        }
    }
    Text {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        text: "Your keyring provider displays its own authentication prompt. Password-based login unlock is configured by your display manager or PAM login stack."
        color: Theme.inkFaint
        font.pixelSize: 12
    }
}
