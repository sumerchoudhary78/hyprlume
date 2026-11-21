import QtQuick
import QtQuick.Controls
import QtQuick.Templates as T
import qs.components
import qs.theme

T.Button {
    id: root
    property string iconName: ""
    property string tooltip: ""
    property bool selected: false
    property bool showChevron: false
    property color iconColor: Theme.inkDim
    property real maximumWidth: Infinity

    implicitHeight: Theme.barHeight
    implicitWidth: Math.max(Theme.px(34), implicitContentWidth + leftPadding + rightPadding)
    leftPadding: Theme.px(8)
    rightPadding: leftPadding
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    Accessible.name: tooltip || text
    opacity: enabled ? 1 : 0.55
    Keys.onReturnPressed: click()
    ToolTip.visible: hovered && !!tooltip
    ToolTip.text: tooltip
    ToolTip.delay: 550

    background: Rectangle {
        anchors.fill: parent
        anchors.topMargin: Theme.px(4)
        anchors.bottomMargin: anchors.topMargin
        radius: Theme.px(7)
        color: root.down ? Theme.pressed : root.hovered || root.selected ? Theme.hover : "transparent"
        border.width: root.visualFocus ? 2 : 0
        border.color: Theme.focusRing
    }
    contentItem: Row {
        id: contents
        spacing: Theme.px(7)
        Icon {
            id: leadingIcon
            anchors.verticalCenter: parent.verticalCenter
            visible: !!root.iconName
            name: root.iconName
            size: Theme.px(16)
            color: root.iconColor
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: !!root.text
            text: root.text
            textFormat: Text.PlainText
            readonly property real available: root.maximumWidth - root.leftPadding - root.rightPadding
                - (leadingIcon.visible ? leadingIcon.width + contents.spacing : 0)
                - (chevron.visible ? chevron.width + contents.spacing : 0)
            width: Math.max(0, Math.min(implicitWidth, Theme.px(142), available))
            elide: Text.ElideRight
            font.family: Theme.fontSans
            font.pixelSize: Theme.px(13)
            font.weight: Font.Medium
            color: Theme.ink
        }
        Icon {
            id: chevron
            anchors.verticalCenter: parent.verticalCenter
            visible: root.showChevron
            size: Theme.px(10)
            name: "chevron-down"; color: Theme.inkDim
        }
    }
    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
