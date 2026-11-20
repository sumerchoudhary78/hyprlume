import QtQuick
import QtQuick.Layouts
import QtQuick.Templates as T
import qs.theme

T.Button {
    id: root

    property string iconName: ""
    property string glyph: ""
    property string label: ""
    property string hint: ""
    property bool active: false
    property bool quiet: false
    property color accent: Theme.cyan

    implicitHeight: Theme.px(36)
    implicitWidth: implicitContentWidth + leftPadding + rightPadding
    leftPadding: Theme.px(10)
    rightPadding: Theme.px(10)
    topPadding: 0
    bottomPadding: 0
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    Accessible.name: label
    Keys.onReturnPressed: click()

    background: Rectangle {
        radius: Theme.px(8)
        color: root.down ? Theme.pressed : root.hovered ? Theme.hover : root.active ? Theme.selection
            : root.quiet ? "transparent" : Theme.control
        border.width: root.visualFocus ? 2 : 1
        border.color: root.visualFocus ? Theme.focusRing : root.active ? Theme.rimStrong
            : root.quiet ? "transparent" : Theme.rim
    }

    contentItem: RowLayout {
        spacing: Theme.px(9)

        Icon {
            name: root.iconName
            glyph: root.glyph
            color: root.active ? root.accent : Theme.inkDim
            size: Theme.px(16)
        }

        Text {
            Layout.fillWidth: true
            text: root.label
            textFormat: Text.PlainText
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontBody
            font.weight: root.active ? Font.DemiBold : Font.Normal
            elide: Text.ElideRight
        }

        Text {
            visible: !!root.hint
            text: root.hint
            textFormat: Text.PlainText
            color: Theme.inkDim
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontCaption
        }
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
