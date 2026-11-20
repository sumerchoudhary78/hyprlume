import QtQuick
import qs.components
import qs.theme

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    Rectangle {
        anchors.fill: parent
        color: "#a0040a0f"
        MouseArea { anchors.fill: parent; onClicked: root.backend.send({op:"toggle"}) }
    }
    CommandPanel {
        transform: PanelEntrance { reducedEffects: root.reducedEffects }
        anchors.centerIn: parent
        width: Math.min(implicitWidth, root.width - Theme.px(32))
        height: Math.min(implicitHeight, root.height - Theme.px(28))
        snapshot: root.backend.snapshot
        connectionError: root.backend.error
        onRequest: operation => root.backend.send(operation)
        onDismissed: root.backend.send({op:"toggle"})
    }
}
