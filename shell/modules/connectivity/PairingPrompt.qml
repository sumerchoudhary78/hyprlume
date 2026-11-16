import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var prompt
    signal replied(bool accept, string value)
    readonly property bool inputNeeded: prompt.kind === "pin" || prompt.kind === "passkey"
    property bool ready: false
    property int shownToken: -1
    function updatePrompt() {
        if (!ready || shownToken === prompt.token) return;
        shownToken = prompt.token;
        value.text = "";
        if (inputNeeded) value.forceActiveFocus();
        else if (prompt.kind === "display") rejectButton.forceActiveFocus();
        else acceptButton.forceActiveFocus();
    }
    onPromptChanged: updatePrompt()
    Component.onCompleted: { ready = true; updatePrompt(); }
    radius: 18; color: "#f0101c36"; border.width: 1; border.color: Theme.violet
    implicitHeight: content.implicitHeight + 28
    ColumnLayout {
        id: content
        anchors.fill: parent; anchors.margins: 14; spacing: 9
        Text {
            text: root.prompt.kind === "confirm" ? "Does this code match your device?" : root.prompt.kind === "display" ? "Enter this code on your Bluetooth device" : root.inputNeeded ? "Enter the device's pairing code" : "Allow this device to connect?"
            color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 14; font.weight: Font.DemiBold
        }
        Text { text: root.prompt.address; textFormat: Text.PlainText; color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 10 }
        Text {
            visible: root.prompt.code.length > 0
            text: root.prompt.code; textFormat: Text.PlainText
            color: Theme.cyan; font.family: Theme.fontMono; font.pixelSize: 29; font.letterSpacing: 6
        }
        Rectangle {
            Layout.fillWidth: true; implicitHeight: 38; visible: root.inputNeeded
            radius: 10; color: Theme.glassInset; border.width: 1; border.color: Theme.rim
            TextInput {
                id: value; objectName: "pairingValue"
                anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 10
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.ink; font.family: Theme.fontMono; font.pixelSize: 16
                maximumLength: root.prompt.kind === "passkey" ? 6 : 16
                onAccepted: { if (text.length > 0) root.replied(true, text); }
                Accessible.name: "Bluetooth pairing code"
            }
        }
        RowLayout {
            GlassButton {
                id: acceptButton
                objectName: "pairingAccept"; primary: true
                visible: root.prompt.kind !== "display"
                text: root.inputNeeded ? "Submit code" : "Confirm"
                enabled: !root.inputNeeded || value.text.length > 0
                onClicked: root.replied(true, value.text)
            }
            GlassButton { id: rejectButton; objectName: "pairingReject"; text: "Cancel pairing"; onClicked: root.replied(false, "") }
            Item { Layout.fillWidth: true }
        }
    }
}
