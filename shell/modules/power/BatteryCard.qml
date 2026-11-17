import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var battery
    property bool reducedEffects: false
    implicitHeight: 132; radius: 20
    color: "#4816253d"; border.width: 1; border.color: "#526381a4"
    function remaining(seconds) {
        const minutes = Math.round(seconds / 60);
        return minutes >= 60 ? Math.floor(minutes / 60) + "h " + minutes % 60 + "m" : minutes + "m";
    }
    RowLayout {
        anchors.fill: parent; anchors.margins: 16; spacing: 16
        Gauge {
            Layout.preferredWidth: 98; Layout.preferredHeight: 98
            label: "ENERGY"; available: root.battery.available && root.battery.present && root.battery.percentage !== null
            value: available ? root.battery.percentage / 100 : 0
            reducedEffects: root.reducedEffects
            accent: available && value < 0.2 ? Theme.amber : Theme.cyan
        }
        ColumnLayout {
            Layout.fillWidth: true; spacing: 7
            Text {
                Layout.fillWidth: true
                text: !root.battery.available ? "Battery unavailable" : !root.battery.present ? "AC-powered desktop" : ({charging:"Charging",discharging:"On battery",empty:"Battery empty",full:"Fully charged",pending_charge:"Waiting to charge",pending_discharge:"Battery ready"}[root.battery.state] || "Battery status unknown")
                color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 18; font.weight: Font.DemiBold
                wrapMode: Text.Wrap
            }
            Text {
                Layout.fillWidth: true
                text: root.battery.seconds_remaining ? root.remaining(root.battery.seconds_remaining) + (root.battery.state === "charging" ? " until full" : " remaining") : root.battery.present ? "Live battery information" : root.battery.available ? "No system battery detected" : "Start UPower to show battery status"
                color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11; wrapMode: Text.Wrap
            }
        }
    }
}
