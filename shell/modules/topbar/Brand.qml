import QtQuick
import qs.components
import qs.theme

Item {
    id: root
    property bool compact: false
    property bool presentation: false
    implicitWidth: row.implicitWidth
    implicitHeight: 40
    Row {
        id: row
        anchors.verticalCenter: parent.verticalCenter
        spacing: 9
        Icon { name: "bolt"; size: 29; color: Theme.violet; anchors.verticalCenter: parent.verticalCenter }
        Text { visible: !root.compact; text: "Hyprlume"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 21; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter }
        Chip { visible: !root.compact; anchors.verticalCenter: parent.verticalCenter; text: root.presentation ? "demo" : "preview"; textColor: Theme.violet; fillColor: "#401e193b"; strokeColor: "transparent" }
    }
}
