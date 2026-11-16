import QtQuick
import QtQuick.Effects
import Quickshell
import qs.theme

// Freedesktop icon from the active icon theme, tinted to the shell palette.
// Falls back to a Nerd Font glyph, then to a quiet dot, so a missing icon in
// someone's theme never leaves a hole in a panel.
Item {
    id: root

    property string name: ""
    property string glyph: ""
    property color color: Theme.ink
    property int size: Theme.px(18)
    property real iconOpacity: 1

    implicitWidth: size
    implicitHeight: size

    readonly property bool hasIcon: root.name.length > 0 && img.status === Image.Ready

    Image {
        id: img
        anchors.fill: parent
        visible: false
        smooth: true
        fillMode: Image.PreserveAspectFit
        source: root.name.length > 0 ? Quickshell.iconPath(root.name, true) : ""
    }

    MultiEffect {
        anchors.fill: img
        source: img
        visible: root.hasIcon
        colorization: 1.0
        colorizationColor: root.color
        opacity: root.iconOpacity
    }

    Text {
        anchors.centerIn: parent
        visible: !root.hasIcon && root.glyph.length > 0
        text: root.glyph
        color: root.color
        opacity: root.iconOpacity
        font.family: Theme.fontSans
        font.pixelSize: root.size
    }

    Rectangle {
        anchors.centerIn: parent
        visible: !root.hasIcon && root.glyph.length === 0
        width: Math.round(root.size * 0.34)
        height: width
        radius: width / 2
        color: root.color
        opacity: root.iconOpacity * 0.4
    }
}
