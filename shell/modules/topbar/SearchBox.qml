import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// Search affordance. It is the entry point to the command panel, so the whole
// field is one button rather than a text input that cannot be typed into.
Item {
    id: root

    signal activated()

    implicitWidth: Theme.px(320)
    implicitHeight: Theme.px(40)

    Rectangle {
        anchors.fill: parent
        radius: Theme.px(13)
        color: mouse.containsMouse ? Qt.rgba(1, 1, 1, 0.08) : Qt.rgba(0.02, 0.05, 0.09, 0.55)
        border.width: 1
        border.color: mouse.containsMouse ? Theme.rimStrong : Theme.divider
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.px(12)
        anchors.rightMargin: Theme.px(6)
        spacing: Theme.px(9)

        Icon {
            name: "system-search"
            size: Theme.px(15)
            color: Theme.inkFaint
        }

        Text {
            Layout.fillWidth: true
            text: "Search or run a command"
            color: Theme.inkFaint
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontBody
            elide: Text.ElideRight
        }

        Chip {
            text: "Super K"
            textColor: Theme.inkDim
            fillColor: Qt.rgba(1, 1, 1, 0.06)
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.activated()
    }
}
