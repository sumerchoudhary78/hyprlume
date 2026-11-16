import QtQuick
import qs.theme

// A static local capture feeds a cached optical shader. Only geometry, style
// or pointer lighting updates the glass; animated water does not re-blur it.
Item {
    id: root
    property real radius: Theme.radiusCard
    property color tint: Theme.glass
    property color tintTop: Theme.glassTop
    property bool glowing: true
    property color glowColor: Theme.cyan
    property color secondaryColor: Theme.violet
    property Item backdrop: null
    property bool reducedEffects: false
    property real refraction: Theme.glassRefraction
    default property alias content: contentItem.data
    readonly property rect captureRect: {
        if (!backdrop) return Qt.rect(0, 0, 1, 1);
        // Register backdrop resize dependencies before mapping coordinates.
        const sceneSize = backdrop.width + backdrop.height;
        const a = backdrop.mapFromItem(root, 0, 0);
        const b = backdrop.mapFromItem(root, root.width, root.height);
        return Qt.rect(a.x, a.y, Math.max(1, b.x - a.x), Math.max(1, b.y - a.y));
    }
    HoverHandler { id: pointer; enabled: !root.reducedEffects }
    ShaderEffectSource {
        id: capture
        sourceItem: root.backdrop
        sourceRect: root.captureRect
        textureSize: Qt.size(Math.max(1, root.width / 2), Math.max(1, root.height / 2))
        live: false
        visible: false
        onSourceRectChanged: scheduleUpdate()
        onSourceItemChanged: scheduleUpdate()
    }
    ShaderEffect {
        id: glass
        anchors.fill: parent
        visible: !root.reducedEffects
        // Cache this background separately from foreground labels/controls.
        layer.enabled: true
        layer.textureSize: Qt.size(Math.max(1, width), Math.max(1, height))
        property var backdropTexture: capture
        property vector2d surfaceSize: Qt.vector2d(width, height)
        property real cornerRadius: root.radius
        property real refraction: root.refraction
        property color tintColor: root.tint
        property color upperTint: root.tintTop
        property color accentA: root.secondaryColor
        property color accentB: root.glowColor
        property vector2d lightPosition: pointer.hovered
            ? Qt.vector2d(pointer.point.position.x / Math.max(root.width, 1), pointer.point.position.y / Math.max(root.height, 1))
            : Qt.vector2d(0.18, 0.08)
        property real hoverAmount: pointer.hovered ? 1 : 0
        property real hasBackdrop: root.backdrop !== null ? 1 : 0
        fragmentShader: "../assets/shaders/liquid-glass.frag.qsb"
    }
    Rectangle {
        anchors.fill: parent
        visible: root.reducedEffects
        radius: root.radius
        border.width: 1; border.color: Theme.rimStrong
        gradient: Gradient {
            GradientStop { position: 0; color: root.tintTop }
            GradientStop { position: 1; color: Theme.glassDeep }
        }
    }
    Canvas {
        id: rim
        anchors.fill: parent; anchors.margins: -10
        visible: root.glowing && !root.reducedEffects
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        Connections {
            target: root
            function onGlowColorChanged() { rim.requestPaint(); }
            function onSecondaryColorChanged() { rim.requestPaint(); }
            function onRadiusChanged() { rim.requestPaint(); }
        }
        onPaint: {
            const c = getContext("2d"), x = 10.5, y = 10.5;
            const w = width - 21, h = height - 21, r = Math.min(root.radius, h / 2);
            c.clearRect(0, 0, width, height);
            const g = c.createLinearGradient(0, 0, width, height);
            g.addColorStop(0, root.secondaryColor);
            g.addColorStop(0.35, "#30405a");
            g.addColorStop(0.8, root.glowColor);
            g.addColorStop(1, "#a4edff");
            c.beginPath();
            c.moveTo(x + r, y); c.lineTo(x + w - r, y);
            c.quadraticCurveTo(x + w, y, x + w, y + r);
            c.lineTo(x + w, y + h - r);
            c.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
            c.lineTo(x + r, y + h);
            c.quadraticCurveTo(x, y + h, x, y + h - r);
            c.lineTo(x, y + r); c.quadraticCurveTo(x, y, x + r, y);
            c.strokeStyle = g;
            for (const pass of [[16, 0.018], [9, 0.035], [4, 0.10], [1.2, 0.50]]) {
                c.globalAlpha = pass[1]; c.lineWidth = pass[0]; c.stroke();
            }
        }
    }
    Connections {
        target: root.backdrop
        ignoreUnknownSignals: true
        function onReadyChanged() { capture.scheduleUpdate(); }
    }
    Item { id: contentItem; anchors.fill: parent }
}
