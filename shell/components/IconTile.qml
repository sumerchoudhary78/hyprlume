import QtQuick
import QtQuick.Controls
import QtQuick.Templates as T
import qs.theme

T.Button {
    id: root

    property string iconName: ""
    property string glyph: ""
    property bool active: false
    property bool quiet: false
    property color accent: Theme.cyan
    property int size: Theme.px(42)
    property int iconSize: Theme.px(19)
    property string tooltip: ""

    implicitWidth: size
    implicitHeight: size
    padding: 0
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    opacity: enabled ? 1 : 0.45
    Accessible.name: tooltip || text
    Keys.onReturnPressed: click()
    ToolTip.visible: hovered && tooltip.length > 0
    ToolTip.delay: 550
    ToolTip.text: tooltip

    background: Rectangle {
        radius: Theme.px(9)
        color: root.down ? Theme.pressed : root.hovered ? Theme.hover : root.active ? Theme.selection
            : root.quiet ? "transparent" : Theme.control
        border.width: root.visualFocus ? 2 : 1
        border.color: root.visualFocus ? Theme.focusRing : root.active ? Theme.rimStrong
            : root.quiet ? "transparent" : Theme.rim
    }

    contentItem: Item {
        Icon {
            anchors.centerIn: parent
            name: root.iconName
            glyph: root.glyph
            size: root.iconSize
            color: root.active ? root.accent : root.hovered ? Theme.ink : Theme.inkDim
        }
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
