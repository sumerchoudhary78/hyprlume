import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    readonly property var snapshot: backend.connectivity
    readonly property bool compact: width < 730
    property int tab: 0
    radius: Theme.radiusPanel
    focus: true
    function send(action) { root.backend.send({op:"connectivity",request:action}); }
    Keys.onEscapePressed: root.send({action:"close"})
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }

    ColumnLayout {
        anchors.fill: parent; anchors.margins: 24; spacing: 16
        RowLayout {
            Layout.fillWidth: true; spacing: 12
            Icon { name: "wifi"; size: 27; color: Theme.cyan }
            Text { text: "Connections"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: Theme.fontHeadline; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            Chip { visible: root.snapshot.busy; text: "Working…"; textColor: Theme.amber }
            IconTile { objectName: "closeConnectivity"; size: 36; iconName: "close"; quiet: true; tooltip: "Close connections"; onClicked: root.send({action:"close"}) }
        }
        Row {
            visible: root.compact; spacing: 8
            GlassButton { text: "Wi-Fi"; compact: true; primary: root.tab === 0; onClicked: root.tab = 0 }
            GlassButton { text: "Bluetooth"; compact: true; primary: root.tab === 1; onClicked: root.tab = 1 }
        }
        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 16
            enabled: root.snapshot.prompt === null
            WifiPane {
                Layout.fillWidth: true; Layout.fillHeight: true
                visible: !root.compact || root.tab === 0
                snapshot: root.snapshot.wifi; busy: root.snapshot.busy
                onRequest: action => root.send(action)
            }
            BluetoothPane {
                Layout.fillWidth: true; Layout.fillHeight: true
                visible: !root.compact || root.tab === 1
                snapshot: root.snapshot.bluetooth; busy: root.snapshot.busy
                onRequest: action => root.send(action)
            }
        }
        Text {
            Layout.fillWidth: true
            text: root.snapshot.error || root.backend.error || root.snapshot.message
            textFormat: Text.PlainText; wrapMode: Text.Wrap
            color: root.snapshot.error || root.backend.error ? Theme.rose : Theme.inkDim
            font.family: Theme.fontSans; font.pixelSize: 11
        }
    }
    Rectangle {
        anchors.fill: parent; radius: root.radius
        visible: root.snapshot.prompt !== null
        color: "#c2040a18"
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    }
    Loader {
        anchors.centerIn: parent
        width: Math.min(520, root.width - 54)
        height: item ? item.implicitHeight : 0
        active: root.snapshot.prompt !== null
        sourceComponent: PairingPrompt {
            prompt: root.snapshot.prompt || ({token:0,kind:"",address:"",code:""})
            onReplied: (accept, value) => root.send({action:"pair_reply",token:prompt.token,accept:accept,value:value.length > 0 ? value : null})
        }
    }
    Component.onCompleted: forceActiveFocus()
}
