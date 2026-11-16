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
    property string adapterId: ""
    readonly property var adapter: snapshot.adapters.find(item => item.id === adapterId) || snapshot.adapters[0] || null
    readonly property var devices: adapter ? snapshot.devices.filter(device => device.adapter === adapter.id) : []
    property string forgetting: ""
    radius: 23; color: "#58051020"; border.width: 1; border.color: "#35b497ff"
    function act(action, device) { root.request({action:action,adapter:device.adapter,address:device.address}); }

    ColumnLayout {
        anchors.fill: parent; anchors.margins: 18; spacing: 13
        RowLayout {
            Layout.fillWidth: true
            Text { text: "Bluetooth"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 21; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            GlassButton {
                objectName: "bluetoothRadio"; compact: true; primary: root.adapter !== null && root.adapter.powered
                text: root.adapter && root.adapter.powered ? "On" : "Off"
                enabled: root.adapter !== null && !root.busy
                onClicked: root.request({action:"bluetooth_power",adapter:root.adapter.id,enabled:!root.adapter.powered})
            }
        }
        RowLayout {
            Layout.fillWidth: true
            Text {
                Layout.fillWidth: true
                text: root.adapter ? root.adapter.discovering ? "Discovering nearby devices…" : root.adapter.name : root.snapshot.error || "No Bluetooth adapter"
                textFormat: Text.PlainText; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11; elide: Text.ElideRight
            }
            GlassButton {
                objectName: "bluetoothScan"; text: root.adapter && root.adapter.discovering ? "Scanning…" : "Scan"; compact: true
                enabled: root.adapter !== null && root.adapter.powered && !root.adapter.discovering && !root.busy
                onClicked: root.request({action:"bluetooth_scan",adapter:root.adapter.id})
            }
        }
        Row {
            visible: root.snapshot.adapters.length > 1; spacing: 6
            Repeater {
                model: root.snapshot.adapters
                GlassButton {
                    required property var modelData
                    text: modelData.id; compact: true; primary: root.adapter !== null && root.adapter.id === modelData.id
                    onClicked: root.adapterId = modelData.id
                }
            }
        }
        ListView {
            id: list
            Layout.fillWidth: true; Layout.fillHeight: true; implicitHeight: 310
            clip: true; spacing: 7; model: root.devices
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Rectangle {
                required property var modelData
                required property int index
                width: list.width; height: 91; radius: 14
                color: "#3e0b182b"; border.width: 1; border.color: modelData.connected ? "#9068f0b1" : "#305f689e"
                ColumnLayout {
                    anchors.fill: parent; anchors.margins: 11; spacing: 5
                    RowLayout {
                        Layout.fillWidth: true
                        Text { Layout.fillWidth: true; text: modelData.name; textFormat: Text.PlainText; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 13; elide: Text.ElideRight }
                        StatusDot { color: modelData.connected ? Theme.mint : modelData.paired ? Theme.violet : Theme.inkFaint; size: 5 }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Text { Layout.fillWidth: true; text: modelData.connected ? "Connected" : modelData.paired ? "Paired" : modelData.address; textFormat: Text.PlainText; color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 9; elide: Text.ElideRight }
                        GlassButton {
                            objectName: "bluetoothAction" + index; compact: true
                            text: modelData.connected ? "Disconnect" : modelData.paired ? "Connect" : "Pair"
                            enabled: !root.busy && root.adapter !== null && root.adapter.powered
                            onClicked: {
                                if (modelData.connected) root.act("bluetooth_disconnect", modelData);
                                else if (modelData.paired) root.act("bluetooth_connect", modelData);
                                else root.request({action:"bluetooth_pair",adapter:modelData.adapter,address:modelData.address,trust:remember.checked});
                            }
                        }
                        GlassButton {
                            objectName: "bluetoothForget" + index; compact: true
                            visible: modelData.paired; text: root.forgetting === modelData.adapter + "/" + modelData.address ? "Confirm" : "Forget"
                            enabled: !root.busy
                            onClicked: {
                                const key = modelData.adapter + "/" + modelData.address;
                                if (root.forgetting === key) { root.forgetting = ""; root.act("bluetooth_forget", modelData); }
                                else root.forgetting = key;
                            }
                        }
                    }
                }
            }
            Text { anchors.centerIn: parent; visible: list.count === 0; text: root.adapter && root.adapter.powered ? "Scan to find your devices." : "Enable Bluetooth to see devices."; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 12 }
        }
        GlassCheckBox {
            id: remember; objectName: "bluetoothTrust"
            text: "Trust new pairings for reconnection"; checked: true; enabled: !root.busy
        }
        Text { text: "Discovery stops after 15 seconds."; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 10 }
    }
}
