import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// Themed button for shell surfaces. Primary uses the same warm fill as the
// active bar mode; everything else stays on the glass palette.
Item {
    id: root

    property string text: ""
    property string iconName: ""
    property bool primary: false
    property bool compact: false

    signal clicked()
    activeFocusOnTab: true
    Accessible.role: Accessible.Button
    Accessible.name: text
    Accessible.onPressAction: { if (root.enabled) root.clicked(); }
    Keys.onReturnPressed: { if (root.enabled) root.clicked(); }
    Keys.onSpacePressed: { if (root.enabled) root.clicked(); }

    implicitHeight: compact ? Theme.px(30) : Theme.px(36)
    implicitWidth: Math.max(implicitHeight, content.implicitWidth + Theme.px(compact ? 20 : 28))
    opacity: enabled ? 1 : 0.42

    Rectangle {
        anchors.fill: parent
        radius: Theme.px(13)
        gradient: Gradient {
            GradientStop {
                position: 0.0
                color: root.primary
                    ? (mouse.pressed ? Qt.rgba(0.87, 0.50, 0.22, 0.95) : Qt.rgba(0.99, 0.73, 0.43, 0.95))
                    : (mouse.containsMouse ? "#60364661" : "#50273750")
            }
            GradientStop {
                position: 1.0
                color: root.primary
                    ? Qt.rgba(0.93, 0.56, 0.26, 0.88)
                    : (mouse.containsMouse ? "#7023324a" : "#700b1426")
            }
        }
        border.width: 1
        border.color: root.primary
            ? Qt.rgba(1.0, 0.88, 0.70, 0.55)
            : (mouse.containsMouse || root.activeFocus ? Theme.rimStrong : Theme.rim)
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2
        radius: Theme.px(11)
        color: "transparent"
        border.width: 1
        border.color: root.primary ? "#35ffffff" : "#1594cced"
    }

    RowLayout {
        id: content
        anchors.centerIn: parent
        spacing: Theme.px(7)

        Icon {
            visible: root.iconName.length > 0
            name: root.iconName
            size: Theme.px(root.compact ? 13 : 15)
            color: root.primary ? "#1b1206" : (mouse.containsMouse ? Theme.ink : Theme.inkDim)
        }

        Text {
            text: root.text
            color: root.primary ? "#1b1206" : (mouse.containsMouse ? Theme.ink : Theme.inkDim)
            font.family: Theme.fontSans
            font.pixelSize: root.compact ? Theme.fontCaption : Theme.fontBody
            font.weight: Font.DemiBold
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        enabled: root.enabled
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
