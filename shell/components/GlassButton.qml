import QtQuick
import QtQuick.Layouts
import QtQuick.Templates as T
import qs.theme

T.Button {
    id: root

    property string iconName: ""
    property bool primary: false
    property bool compact: false
    property bool quiet: false
    property color accent: Theme.cyan

    implicitHeight: Theme.px(compact ? 32 : 40)
    implicitWidth: Math.max(implicitHeight, implicitContentWidth + Theme.px(compact ? 20 : 28))
    padding: 0
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    Keys.onReturnPressed: click()

    background: Rectangle {
        radius: Theme.px(root.compact ? 8 : 10)
        color: !root.enabled ? Theme.control : root.primary
            ? (root.down ? Qt.darker(root.accent, 1.18) : root.hovered ? Qt.lighter(root.accent, 1.08) : root.accent)
            : root.down ? Theme.pressed : root.hovered ? Theme.hover : root.quiet ? "transparent" : Theme.control
        border.width: root.visualFocus ? 2 : 1
        border.color: root.visualFocus ? (root.primary ? Theme.onAccent : Theme.focusRing)
            : root.primary || root.quiet ? "transparent" : Theme.rim
    }

    contentItem: Item {
        implicitWidth: content.implicitWidth
        implicitHeight: content.implicitHeight

        RowLayout {
            id: content
            anchors.centerIn: parent
            spacing: Theme.px(7)

            Icon {
                visible: !!root.iconName
                name: root.iconName
                size: Theme.px(root.compact ? 15 : 17)
                color: root.primary && root.enabled ? Theme.onAccent : Theme.inkDim
            }

            Text {
                visible: !!root.text
                text: root.text
                textFormat: Text.PlainText
                color: !root.enabled ? Theme.inkFaint : root.primary ? Theme.onAccent : Theme.ink
                font.family: Theme.fontSans
                font.pixelSize: root.compact ? Theme.fontCaption : Theme.fontBody
                font.weight: Font.DemiBold
            }
        }
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
