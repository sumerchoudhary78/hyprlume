import QtQuick
import qs.theme

// Square icon button used by the workspace rail and the action ring.
Item {
    id: root

    property string iconName: ""
    property string glyph: ""
    property bool active: false
    property color accent: Theme.amber
    property int size: Theme.px(46)
    property int iconSize: Theme.px(20)
    property string tooltip: ""

    signal clicked()

    implicitWidth: size
    implicitHeight: size

    Rectangle {
        anchors.centerIn: parent
        visible: root.active
        width: parent.width + Theme.px(8)
        height: width
        radius: width / 2
        color: Qt.rgba(root.accent.r, root.accent.g, root.accent.b, 0.12)
    }

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusTile
        color: root.active
            ? Qt.rgba(root.accent.r, root.accent.g, root.accent.b, 0.9)
            : (mouse.containsMouse ? Theme.hover : Qt.rgba(1, 1, 1, 0.035))
        border.width: 1
        border.color: root.active
            ? Qt.rgba(root.accent.r, root.accent.g, root.accent.b, 0.9)
            : Theme.divider

        Icon {
            anchors.centerIn: parent
            name: root.iconName
            glyph: root.glyph
            size: root.iconSize
            color: root.active ? "#101820" : (mouse.containsMouse ? Theme.ink : Theme.inkDim)
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
