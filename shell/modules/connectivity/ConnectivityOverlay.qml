import QtQuick

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    Rectangle {
        anchors.fill: parent; color: "#a0050b18"
        MouseArea { anchors.fill: parent; onClicked: root.backend.send({op:"connectivity",request:{action:"close"}}) }
    }
    ConnectivityPanel {
        anchors.centerIn: parent
        width: Math.min(980, root.width - 32)
        height: Math.min(760, root.height - 28)
        backend: root.backend
        reducedEffects: root.reducedEffects
    }
}
