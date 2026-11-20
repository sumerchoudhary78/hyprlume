import QtQuick
import qs.theme

Item {
    id: root
    property real radius: Theme.radiusCard
    property color tint: Theme.glass
    property color tintTop: Theme.glassTop
    property Item backdrop: null
    property bool reducedEffects: false
    property real refraction: Theme.glassRefraction
    default property alias content: contentItem.data

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        border.width: 1
        border.color: Theme.rim
        gradient: Gradient {
            GradientStop { position: 0; color: Theme.highContrast ? Theme.glassTop : root.tintTop }
            GradientStop { position: 1; color: Theme.highContrast ? Theme.glassDeep : root.tint }
        }
    }

    // The command surface can take a restrained, static sample of the wallpaper.
    // Regular cards and overlays never create the off-screen texture.
    Loader {
        anchors.fill: parent
        active: root.visible && root.backdrop !== null && !root.reducedEffects && !Theme.highContrast && root.refraction > 0
        sourceComponent: Item {
            ShaderEffectSource {
                id: capture
                sourceItem: root.backdrop
                sourceRect: {
                    const backdrop = root.backdrop;
                    if (!backdrop || backdrop.width <= 0 || backdrop.height <= 0)
                        return Qt.rect(0, 0, 1, 1);
                    const origin = backdrop.mapFromItem(root, 0, 0);
                    return Qt.rect(origin.x, origin.y, Math.max(1, root.width), Math.max(1, root.height));
                }
                textureSize: Qt.size(Math.max(1, root.width / 2), Math.max(1, root.height / 2))
                live: false
                visible: false
                onSourceRectChanged: scheduleUpdate()
                onSourceItemChanged: scheduleUpdate()
            }

            ShaderEffect {
                anchors.fill: parent
                opacity: 0.28
                property var backdropTexture: capture
                property vector2d surfaceSize: Qt.vector2d(width, height)
                property real cornerRadius: root.radius
                property real refraction: root.refraction
                property color tintColor: root.tint
                property color upperTint: root.tintTop
                property color accentA: Theme.cyan
                property color accentB: Theme.inkFaint
                property vector2d lightPosition: Qt.vector2d(0.18, 0.08)
                property real hoverAmount: 0
                property real hasBackdrop: 1
                fragmentShader: "../assets/shaders/liquid-glass.frag.qsb"
            }

            Connections {
                target: root.backdrop
                function onReadyChanged() { capture.scheduleUpdate(); }
            }
        }
    }

    Item { id: contentItem; anchors.fill: parent }
}
