import QtQuick
import qs.theme

// Small rounded label used for git branches, versions, counts, shortcuts.
Rectangle {
    id: root

    property string text: ""
    property color textColor: Theme.inkDim
    property color fillColor: Qt.rgba(1, 1, 1, 0.06)
    property color strokeColor: "transparent"
    property bool mono: false
    property int horizontalPadding: Theme.px(9)

    implicitWidth: label.implicitWidth + horizontalPadding * 2
    implicitHeight: Theme.px(22)
    radius: Theme.radiusPill
    color: fillColor
    border.width: strokeColor.a > 0 ? 1 : 0
    border.color: strokeColor

    Text {
        id: label
        anchors.centerIn: parent
        text: root.text
        color: root.textColor
        font.family: root.mono ? Theme.fontMono : Theme.fontSans
        font.pixelSize: Theme.fontCaption
        font.weight: Font.Medium
    }
}
