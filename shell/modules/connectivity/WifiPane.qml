import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var snapshot
    property bool busy: false
    signal request(var action)
    property string selectedId: ""
    property string forgetting: ""
    readonly property var selected: snapshot.networks.find(network => network.id === selectedId) || null
    readonly property bool passwordNeeded: selected !== null && ["wpa2", "wpa3"].indexOf(selected.security) >= 0 && (!selected.saved_uuid || overridePassword.checked)
    onSelectedIdChanged: { password.text = ""; overridePassword.checked = false; forgetting = ""; }
    onSnapshotChanged: { if (!snapshot.enabled || !snapshot.networks.some(network => network.id === selectedId)) selectedId = ""; }
    radius: 23; color: "#58051020"
    border.width: 1; border.color: "#315ce1ff"

    ColumnLayout {
        anchors.fill: parent; anchors.margins: 18; spacing: 13
        RowLayout {
            Layout.fillWidth: true
            Text { text: "Wi-Fi"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 21; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            GlassButton {
                objectName: "wifiRadio"
                compact: true; primary: root.snapshot.enabled
                text: root.snapshot.enabled ? "On" : "Off"
                enabled: root.snapshot.available && !root.busy && (root.snapshot.enabled || root.snapshot.hardware_enabled)
                onClicked: root.request({action:"wifi_power",enabled:!root.snapshot.enabled})
            }
        }
        RowLayout {
            Layout.fillWidth: true
            Text {
                Layout.fillWidth: true
                text: !root.snapshot.available ? (root.snapshot.error || "No Wi-Fi adapter") : !root.snapshot.hardware_enabled ? "Blocked by the hardware switch" : root.snapshot.enabled ? "Networks around your workspace" : "Wi-Fi radio is off"
                textFormat: Text.PlainText; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11; elide: Text.ElideRight
            }
            GlassButton {
                objectName: "wifiScan"; text: "Scan"; compact: true
                enabled: root.snapshot.available && root.snapshot.enabled && !root.busy
                onClicked: root.request({action:"wifi_scan",interface:null})
            }
        }
        ListView {
            id: networks
            Layout.fillWidth: true; Layout.fillHeight: true
            implicitHeight: 310; clip: true; spacing: 7
            model: root.snapshot.enabled ? root.snapshot.networks : []
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Rectangle {
                required property var modelData
                required property int index
                objectName: "wifiNetwork" + index
                width: networks.width; height: 68; radius: 14
                color: root.selectedId === modelData.id ? "#62405775" : hover.hovered ? "#402a405b" : "#3e0b182b"
                border.width: 1; border.color: modelData.connected ? "#9068f0b1" : root.selectedId === modelData.id ? Theme.cyan : "#235f8ba8"
                HoverHandler { id: hover }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.selectedId = modelData.id }
                RowLayout {
                    anchors.fill: parent; anchors.margins: 12; spacing: 10
                    Icon { name: "wifi"; size: 20; color: modelData.connected ? Theme.mint : Theme.cyan }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 5
                        Text { Layout.fillWidth: true; text: modelData.ssid; textFormat: Text.PlainText; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 13; font.weight: Font.Medium; elide: Text.ElideRight }
                        Text { text: (modelData.connected ? "Connected" : modelData.saved_uuid ? "Saved" : modelData.security.toUpperCase()) + "  ·  " + (modelData.frequency >= 5900 ? "6 GHz" : modelData.frequency >= 4900 ? "5 GHz" : "2.4 GHz"); color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 10 }
                    }
                    Text { text: modelData.strength + "%"; color: modelData.connected ? Theme.mint : Theme.inkDim; font.family: Theme.fontMono; font.pixelSize: 11 }
                }
            }
            Text { anchors.centerIn: parent; visible: networks.count === 0; text: root.snapshot.enabled ? "No networks found. Try a scan." : "Enable Wi-Fi to see networks."; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 12 }
        }
        ColumnLayout {
            visible: root.selected !== null
            Layout.fillWidth: true; spacing: 10
            Text { Layout.fillWidth: true; text: root.selected ? root.selected.ssid : ""; textFormat: Text.PlainText; color: Theme.cyan; font.family: Theme.fontSans; font.pixelSize: 14; elide: Text.ElideRight }
            GlassCheckBox {
                id: overridePassword
                visible: root.selected !== null && root.selected.saved_uuid !== null && ["wpa2","wpa3"].indexOf(root.selected.security) >= 0 && !root.selected.connected
                text: "Use a different password"; enabled: !root.busy
            }
            Rectangle {
                Layout.fillWidth: true; implicitHeight: 40
                visible: root.passwordNeeded && !root.selected.connected
                radius: 10; color: Theme.glassInset; border.width: 1; border.color: password.activeFocus ? Theme.cyan : Theme.rim
                TextInput {
                    id: password; objectName: "wifiPassword"
                    anchors.fill: parent; anchors.leftMargin: 12; anchors.rightMargin: 12
                    verticalAlignment: TextInput.AlignVCenter
                    echoMode: TextInput.Password; maximumLength: 128; selectByMouse: true
                    color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 13
                    enabled: !root.busy
                    onAccepted: connect.clicked()
                    Accessible.name: "Wi-Fi password"
                    Text { anchors.verticalCenter: parent.verticalCenter; visible: password.text.length === 0; text: "Network password"; color: Theme.inkFaint; font: password.font }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                GlassButton {
                    id: connect; objectName: "wifiConnect"
                    primary: true
                    text: root.selected && root.selected.connected ? "Disconnect" : "Connect"
                    enabled: root.selected !== null && !root.busy && (!root.passwordNeeded || password.text.length > 0)
                    onClicked: {
                        if (!enabled || !root.selected) return;
                        if (root.selected.connected) root.request({action:"wifi_disconnect",interface:root.selected.interface});
                        else root.request({action:"wifi_connect",network:root.selected.id,password:root.passwordNeeded ? password.text : null});
                        password.text = "";
                    }
                }
                GlassButton {
                    objectName: "wifiForget"; text: root.selected && root.forgetting === root.selected.saved_uuid ? "Forget now" : "Forget"; compact: true
                    visible: root.selected !== null && root.selected.saved_uuid !== null
                    enabled: !root.busy
                    onClicked: {
                        if (root.forgetting === root.selected.saved_uuid) { root.forgetting = ""; root.request({action:"wifi_forget",uuid:root.selected.saved_uuid}); }
                        else root.forgetting = root.selected.saved_uuid;
                    }
                }
                Item { Layout.fillWidth: true }
            }
        }
    }
}
