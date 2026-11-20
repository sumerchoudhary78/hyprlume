import QtQuick
import qs.components

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    Rectangle {
        anchors.fill: parent; color: "#a0050b18"
        MouseArea { anchors.fill: parent; onClicked: root.backend.send({op:"connectivity",request:{action:"close"}}) }
    }
    ConnectivityPanel {
        transform: PanelEntrance { reducedEffects: root.reducedEffects }
        anchors.centerIn: parent
        width: Math.min(880, root.width - 32)
        height: Math.min(574, root.height - 28)
        backend: root.backend
        reducedEffects: root.reducedEffects
    }
}
