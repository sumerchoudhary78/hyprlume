import QtQuick
import QtQuick.Effects
import Quickshell.Widgets
import qs.theme

// The scene supplies a wallpaper item. Geometry changes refresh this small,
// half-resolution capture; text and metrics never re-blur the whole desktop.
Item {
    id: root
    property real radius: Theme.radiusCard
    property color tint: Theme.glass
    property color tintTop: Theme.glassTop
    property color rimColor: Theme.cyan
    property bool glowing: true
    property color glowColor: Theme.cyan
    property color secondaryColor: Theme.violet
    property Item backdrop: null
    property bool reducedEffects: false
    default property alias content: contentItem.data

    readonly property rect captureRect: {
        if (!backdrop) return Qt.rect(0, 0, 1, 1);
        const sceneSize = backdrop.width + backdrop.height;
        const a = backdrop.mapFromItem(root, 0, 0);
        const b = backdrop.mapFromItem(root, root.width, root.height);
        return Qt.rect(a.x, a.y, Math.max(1, b.x - a.x), Math.max(1, b.y - a.y));
    }

    Canvas {
        id: rim
        z: 1
        anchors.fill: parent
        anchors.margins: -12
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        Connections {
            target: root
            function onGlowColorChanged() { rim.requestPaint(); }
            function onSecondaryColorChanged() { rim.requestPaint(); }
        }
        onPaint: {
            const c = getContext("2d"), x = 12.5, y = 12.5;
            const w = width - 25, h = height - 25, r = Math.min(root.radius, h / 2);
            c.clearRect(0, 0, width, height);
            const g = c.createLinearGradient(0, 0, width, height);
            g.addColorStop(0, root.secondaryColor);
            g.addColorStop(0.28, "#4e536e");
            g.addColorStop(0.55, "#255b7f");
            g.addColorStop(0.75, root.glowColor);
            g.addColorStop(1, root.glowColor);
            c.beginPath();
            c.moveTo(x + r, y); c.lineTo(x + w - r, y);
            c.quadraticCurveTo(x + w, y, x + w, y + r);
            c.lineTo(x + w, y + h - r);
            c.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
            c.lineTo(x + r, y + h);
            c.quadraticCurveTo(x, y + h, x, y + h - r);
            c.lineTo(x, y + r); c.quadraticCurveTo(x, y, x + r, y);
            c.strokeStyle = g;
            if (root.glowing && !root.reducedEffects) {
                for (const pass of [[22, 0.025], [14, 0.05], [8, 0.09], [4, 0.22], [2, 0.62]]) {
                    c.globalAlpha = pass[1]; c.lineWidth = pass[0]; c.stroke();
                }
            }
            c.globalAlpha = 0.95; c.lineWidth = 1.0; c.stroke();
        }
    }

    ClippingRectangle {
        anchors.fill: parent
        radius: root.radius
        color: "transparent"
        ShaderEffectSource {
            id: capture
            anchors.fill: parent
            sourceItem: root.backdrop
            sourceRect: root.captureRect
            textureSize: Qt.size(Math.max(1, root.width / 2), Math.max(1, root.height / 2))
            live: false
            visible: false
            onSourceRectChanged: scheduleUpdate()
            onSourceItemChanged: scheduleUpdate()
        }
        MultiEffect {
            anchors.fill: parent
            visible: root.backdrop !== null && !root.reducedEffects
            source: capture
            blurEnabled: true
            blurMax: 24
            blur: 0.8
            saturation: -0.15
        }
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0; color: root.tintTop }
                GradientStop { position: 0.48; color: root.tint }
                GradientStop { position: 1; color: Theme.glassDeep }
            }
        }
        Rectangle {
            width: parent.width
            height: parent.height * 0.52
            rotation: -14
            y: -height * 0.55
            gradient: Gradient {
                GradientStop { position: 0; color: "#14ffffff" }
                GradientStop { position: 1; color: "#00ffffff" }
            }
        }
    }
    Connections {
        target: root.backdrop
        ignoreUnknownSignals: true
        function onReadyChanged() { capture.scheduleUpdate(); }
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: 2
        radius: Math.max(0, root.radius - 2)
        color: "transparent"
        border.width: 1
        border.color: "#13d9eaff"
    }
    Item { id: contentItem; anchors.fill: parent; z: 2 }
}
