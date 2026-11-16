import QtQuick
import QtQuick.Layouts
import qs.theme

// One line of a list: icon, label, optional right-aligned hint.
Item {
    id: root

    property string iconName: ""
    property string glyph: ""
    property string label: ""
    property string hint: ""
    property bool active: false
    property color accent: Theme.cyan

    signal clicked()

    implicitHeight: Theme.px(31)

    Rectangle {
        anchors.fill: parent
        radius: Theme.px(12)
        gradient: Gradient {
            GradientStop { position: 0; color: mouse.containsMouse || root.active ? "#68516a8a" : "#3c32435e" }
            GradientStop { position: 0.48; color: mouse.containsMouse || root.active ? "#70233550" : "#580d1b31" }
            GradientStop { position: 1; color: mouse.containsMouse || root.active ? "#603b426b" : "#3a25273f" }
        }
        border.width: 1
        border.color: root.active ? root.accent : "#1c99c9ed"
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.px(10)
        anchors.rightMargin: Theme.px(10)
        spacing: Theme.px(10)

        Icon {
            name: root.iconName
            glyph: root.glyph
            color: root.active || mouse.containsMouse ? root.accent : Theme.inkDim
            size: Theme.px(16)
        }

        Text {
            Layout.fillWidth: true
            text: root.label
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontBody
            elide: Text.ElideRight
        }

        Text {
            visible: root.hint.length > 0
            text: root.hint
            color: Theme.inkFaint
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontCaption
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
