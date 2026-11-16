import QtQuick
import qs.theme

// The glass surface every panel is built on.
//
// Compositor blur is requested by the *window* that hosts this item, not here
// (see the PanelWindow declarations in modules/). This item owns the painted
// part: tint gradient, rim, top sheen, and an optional accent bloom.
Item {
    id: root

    property real radius: Theme.radiusCard
    property color tint: Theme.glass
    property color tintTop: Theme.glassTop
    property color rimColor: Theme.rim
    property bool glowing: false
    property color glowColor: Theme.cyan

    default property alias content: contentItem.data

    // Accent bloom. Two flat rectangles are cheaper than a shader and enough
    // for the soft outline the reference uses.
    Rectangle {
        visible: root.glowing
        anchors.fill: parent
        anchors.margins: -Theme.px(2)
        radius: root.radius + Theme.px(2)
        color: Qt.rgba(root.glowColor.r, root.glowColor.g, root.glowColor.b, 0.07)
        border.width: 1
        border.color: Qt.rgba(root.glowColor.r, root.glowColor.g, root.glowColor.b, 0.22)
    }

    Rectangle {
        id: body
        anchors.fill: parent
        radius: root.radius
        border.width: Theme.borderWidth
        border.color: root.rimColor
        gradient: Gradient {
            GradientStop { position: 0.0; color: root.tintTop }
            GradientStop { position: 0.42; color: root.tint }
            GradientStop { position: 1.0; color: Theme.glassDeep }
        }
    }

    // Top sheen: a full-size gradient rather than a clipped strip, so the
    // rounded corners stay exact.
    Rectangle {
        anchors.fill: parent
        radius: root.radius
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.rgba(1, 1, 1, 0.075) }
            GradientStop { position: 0.34; color: Qt.rgba(1, 1, 1, 0.0) }
            GradientStop { position: 1.0; color: Qt.rgba(1, 1, 1, 0.0) }
        }
    }

    Item {
        id: contentItem
        anchors.fill: parent
    }
}
