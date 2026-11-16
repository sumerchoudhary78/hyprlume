import QtQuick
import qs.theme

Item {
    id: root
    required property var backend
    Rectangle {
        anchors.fill: parent
        color: "#96020815"
        MouseArea { anchors.fill: parent; onClicked: root.backend.send({op:"toggle"}) }
    }
    CommandPanel {
        anchors.centerIn: parent
        width: Math.min(640, root.width - 40)
        height: Math.min(implicitHeight, root.height - 32)
        snapshot: root.backend.snapshot
        connectionError: root.backend.error
        onRequest: operation => root.backend.send(operation)
        onDismissed: root.backend.send({op:"toggle"})
    }
}
