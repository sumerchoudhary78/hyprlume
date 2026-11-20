import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    readonly property var status: backend.snapshot.storage || {}
    property string error: ""
    property var review: null
    property int pending: 0

    function close() {
        review = null;
        backend.send({op: "panel", panel: "storage", open: false});
    }

    function send(request) {
        error = "";
        pending = backend.send({op: "storage", request: request}) || 0;
        if (!pending) error = backend.error;
    }

    function submitReview() {
        if (!root.review) return;
        const request = {action: root.review.action, "id": root.review.id};
        if (request.action === "unlock") {
            request.password = passphrase.text;
            passphrase.clear();
        }
        review = null;
        send(request);
    }

    Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (id !== root.pending) return;
            root.pending = 0;
            if (!ok) root.error = error;
        }
    }

    focus: true
    Component.onCompleted: forceActiveFocus()
    Keys.onEscapePressed: {
        if (review) { passphrase.clear(); review = null; }
        else close();
    }

    Rectangle {
        anchors.fill: parent
        color: "#98050b18"
        MouseArea { anchors.fill: parent; onClicked: root.close() }
    }

    GlassSurface {
        transform: PanelEntrance { reducedEffects: root.reducedEffects }
        anchors.centerIn: parent
        width: Math.min(690, parent.width - 32)
        height: Math.min(600, parent.height - 32)
        radius: Theme.radiusPanel
        reducedEffects: root.reducedEffects

        MouseArea { anchors.fill: parent }
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 21
            spacing: 14

            RowLayout {
                Layout.fillWidth: true
                Icon { name: "drive"; color: Theme.cyan; size: 24 }
                Text { Layout.fillWidth: true; text: "Removable drives"; color: Theme.ink; font.pixelSize: 24; font.family: Theme.fontSans }
                GlassButton { text: "Refresh"; compact: true; enabled: !root.pending && !root.status.busy && !root.review; onClicked: root.send({action: "refresh"}) }
                IconTile { objectName: "closeStorage"; iconName: "close"; size: 30; onClicked: root.close() }
            }
            Text {
                Layout.fillWidth: true
                text: root.error || root.status.error || (root.status.busy ? "Waiting for the storage service…" : root.status.message) || "Reading removable drives…"
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                color: root.error || root.status.error ? Theme.amber : Theme.inkDim
                font.pixelSize: 13
            }
            ScrollView {
                id: scroll
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentWidth: availableWidth
                clip: true
                enabled: !root.review && !root.pending && !root.status.busy
                ColumnLayout {
                    width: scroll.availableWidth
                    spacing: 12
                    Repeater {
                        model: root.status.drives || []
                        ColumnLayout {
                            id: drive
                            required property var modelData
                            required property int index
                            Layout.fillWidth: true
                            spacing: 10
                            RowLayout {
                                Layout.fillWidth: true
                                Text {
                                    Layout.fillWidth: true
                                    text: drive.modelData.label
                                    textFormat: Text.PlainText
                                    elide: Text.ElideRight
                                    color: Theme.cyan
                                    font.pixelSize: 15
                                }
                                GlassButton {
                                    objectName: "driveEject" + drive.index
                                    text: "Safely remove…"
                                    compact: true
                                    enabled: drive.modelData.can_eject
                                    onClicked: root.review = {"id": drive.modelData.id, action: "eject", label: drive.modelData.label}
                                }
                            }
                            Repeater {
                                model: drive.modelData.volumes
                                ColumnLayout {
                                    id: volume
                                    required property var modelData
                                    required property int index
                                    Layout.fillWidth: true
                                    spacing: 5
                                    Text {
                                        Layout.fillWidth: true
                                        text: volume.modelData.label
                                        textFormat: Text.PlainText
                                        elide: Text.ElideRight
                                        color: Theme.ink
                                        font.pixelSize: 13
                                    }
                                    Flow {
                                        Layout.fillWidth: true
                                        spacing: 6
                                        enabled: !volume.modelData.protected
                                        GlassButton {
                                            objectName: "driveMount" + drive.index + "_" + volume.index
                                            visible: volume.modelData.filesystem
                                            text: volume.modelData.mounted ? "Unmount" : "Mount"
                                            compact: true
                                            onClicked: root.send({action: volume.modelData.mounted ? "unmount" : "mount", "id": volume.modelData.id})
                                        }
                                        GlassButton {
                                            objectName: "driveOpen" + drive.index + "_" + volume.index
                                            visible: volume.modelData.mounted
                                            text: "Open files"
                                            compact: true
                                            onClicked: root.send({action: "open", "id": volume.modelData.id})
                                        }
                                        GlassButton {
                                            objectName: "driveUnlock" + drive.index + "_" + volume.index
                                            visible: volume.modelData.encrypted
                                            enabled: !volume.modelData.mounted
                                            text: volume.modelData.locked ? "Unlock…" : "Lock"
                                            compact: true
                                            onClicked: {
                                                if (volume.modelData.locked) {
                                                    root.review = {"id": volume.modelData.id, action: "unlock", label: volume.modelData.label};
                                                    passphrase.forceActiveFocus();
                                                } else root.send({action: "lock", "id": volume.modelData.id});
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true
                visible: !!root.review
                Text {
                    Layout.fillWidth: true
                    text: root.review?.action === "unlock" ? "Unlock " + root.review.label : "Safely remove " + (root.review?.label || "") + "? Close any open files first."
                    textFormat: Text.PlainText
                    wrapMode: Text.Wrap
                    color: Theme.ink
                    font.pixelSize: 14
                }
                TextField {
                    id: passphrase
                    objectName: "drivePassphrase"
                    Layout.fillWidth: true
                    Layout.preferredHeight: 38
                    visible: root.review?.action === "unlock"
                    placeholderText: "Drive passphrase"
                    echoMode: TextInput.Password
                    inputMethodHints: Qt.ImhHiddenText | Qt.ImhSensitiveData | Qt.ImhNoPredictiveText
                    maximumLength: 256
                    selectByMouse: false
                    color: Theme.ink
                    background: Rectangle { radius: 10; color: "#92102136"; border.color: passphrase.activeFocus ? Theme.cyan : Theme.rim }
                    onAccepted: if (length) root.submitReview()
                }
                RowLayout {
                    Layout.fillWidth: true
                    Item { Layout.fillWidth: true }
                    GlassButton { text: "Cancel"; onClicked: { passphrase.clear(); root.review = null; } }
                    GlassButton {
                        objectName: "driveConfirm"
                        text: root.review?.action === "unlock" ? "Unlock drive" : "Remove drive"
                        primary: true
                        enabled: root.review?.action !== "unlock" || passphrase.length > 0
                        onClicked: root.submitReview()
                    }
                }
            }
        }
    }
}
