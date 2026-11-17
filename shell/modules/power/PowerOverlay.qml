import QtQuick
import qs.theme

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    Rectangle {
        anchors.fill: parent; color: "#80050b18"
        MouseArea { anchors.fill: parent; onClicked: root.backend.send({op:"panel",panel:"power",open:false}) }
    }
    PowerPanel {
        anchors.right: parent.right; anchors.top: parent.top
        anchors.rightMargin: 16; anchors.topMargin: Math.min(Theme.barZone + 12, 86)
        width: Math.min(460, root.width - 32)
        height: Math.max(250, root.height - y - 18)
        backend: root.backend; reducedEffects: root.reducedEffects
    }
}
