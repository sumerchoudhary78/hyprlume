import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// One entry of the bar's mode strip. The active entry carries the warm fill
// from the reference; the rest stay quiet until hovered.
Item {
    id: root

    property string label: ""
    property string iconName: ""
    property bool active: false
    property bool showLabel: true

    signal clicked()

    implicitWidth: content.implicitWidth + Theme.px(22)
    implicitHeight: Theme.px(38)

    Rectangle {
        anchors.fill: parent
        radius: Theme.px(12)
        gradient: Gradient {
            GradientStop {
                position: 0.0
                color: root.active
                    ? Qt.rgba(0.99, 0.73, 0.43, 0.95)
                    : (mouse.containsMouse ? Theme.hover : "transparent")
            }
            GradientStop {
                position: 1.0
                color: root.active
                    ? Qt.rgba(0.93, 0.56, 0.26, 0.88)
                    : (mouse.containsMouse ? Theme.hover : "transparent")
            }
        }
        border.width: root.active ? 1 : 0
        border.color: Qt.rgba(1.0, 0.88, 0.70, 0.55)
    }

    RowLayout {
        id: content
        anchors.centerIn: parent
        spacing: Theme.px(8)

        Icon {
            name: root.iconName
            size: Theme.px(15)
            color: root.active ? "#1b1206" : (mouse.containsMouse ? Theme.ink : Theme.inkDim)
        }

        Text {
            visible: root.showLabel
            text: root.label
            color: root.active ? "#1b1206" : (mouse.containsMouse ? Theme.ink : Theme.inkDim)
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontBody
            font.weight: root.active ? Font.DemiBold : Font.Medium
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
