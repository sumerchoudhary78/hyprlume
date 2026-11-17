import QtQuick
import qs.components
import qs.theme

Item {
    id: root
    required property var controller
    required property var trayItem
    implicitWidth: 34; implicitHeight: 34
    signal activated()
    Image {
        id: image
        anchors.centerIn: parent; width: 24; height: 24
        source: root.controller.iconSource(root.trayItem.icon)
        sourceSize: Qt.size(32, 32); asynchronous: true
    }
    Icon { anchors.centerIn: parent; visible: image.status !== Image.Ready; name: "grid"; size: 22; color: Theme.inkDim }
    StatusDot { anchors.right: parent.right; anchors.top: parent.top; size: 5; visible: root.trayItem.status === 2; color: Theme.amber }
    Accessible.name: root.controller.label(root.trayItem.title || root.trayItem.id, "Application")
    MouseArea {
        anchors.fill: parent; acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: event => {
            if (event.button === Qt.RightButton) root.controller.open(root.trayItem);
            else if (root.controller.activate(root.trayItem, event.button === Qt.MiddleButton)) root.activated();
        }
        onWheel: event => root.controller.scroll(root.trayItem, event.angleDelta.y || event.angleDelta.x, event.angleDelta.y === 0)
    }
}
