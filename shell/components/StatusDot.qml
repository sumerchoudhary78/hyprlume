import QtQuick
import qs.theme

Item {
    id: root

    property color color: Theme.mint
    property bool halo: true
    property int size: Theme.px(8)

    implicitWidth: size
    implicitHeight: size

    Rectangle {
        anchors.centerIn: parent
        visible: root.halo
        width: root.size * 2.4
        height: width
        radius: width / 2
        color: root.color
        opacity: 0.18
    }

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: root.color
    }
}
