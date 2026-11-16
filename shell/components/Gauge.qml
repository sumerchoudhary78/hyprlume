import QtQuick
import Quickshell.Widgets
import qs.theme

// The water's area represents the fraction, including near 0% and 100%.
// The fragment shader owns waves and optics; labels remain native crisp text.
Item {
    id: root
    property real value: 0
    property bool available: true
    property color accent: Theme.cyan
    property string label: ""
    property string valueText: hasValue ? Math.round(fraction * 100) + "%" : "—"
    property int diameter: Theme.px(88)
    property bool reducedEffects: false
    property bool showLabels: true
    property var motionClock: null
    property real phaseOffset: 0
    readonly property bool hasValue: available && Number.isFinite(value)
    readonly property real fraction: hasValue ? Math.max(0, Math.min(1, value)) : 0
    readonly property real targetLevel: levelForFraction(fraction)
    readonly property bool motionEnabled: motionClock !== null && motionClock.active && !motionClock.reducedMotion && !reducedEffects
    property real displayedLevel: 0
    property bool initialized: false
    implicitWidth: diameter
    implicitHeight: diameter

    function levelForFraction(fraction) {
        if (fraction <= 0) return 0;
        if (fraction >= 1) return 1;
        // Invert a circular segment's area, once per reading, not per pixel.
        let low = -1, high = 1;
        for (let i = 0; i < 18; i++) {
            const cut = (low + high) / 2;
            const area = (Math.acos(cut) - cut * Math.sqrt(1 - cut * cut)) / Math.PI;
            if (area > fraction) low = cut; else high = cut;
        }
        return (1 - (low + high) / 2) / 2;
    }
    onFractionChanged: {
        if (!initialized || !motionEnabled) displayedLevel = levelForFraction(fraction);
        else motionClock.pulse();
    }
    onMotionEnabledChanged: { if (!motionEnabled) displayedLevel = targetLevel; }
    Connections {
        target: root.motionClock
        function onAdvanced(seconds) {
            const delta = root.targetLevel - root.displayedLevel;
            root.displayedLevel = Math.abs(delta) < 0.0005 ? root.targetLevel
                : root.displayedLevel + delta * (1 - Math.exp(-seconds * 7));
        }
    }
    Item {
        id: orb
        anchors.centerIn: parent
        width: Math.min(root.width, root.height)
        height: width
        ShaderEffect {
            id: water
            anchors.fill: parent
            visible: !root.reducedEffects
            property vector2d surfaceSize: Qt.vector2d(width, height)
            property color liquidColor: root.accent
            property real fillHeight: root.displayedLevel
            property real phase: root.motionClock && root.hasValue && root.fraction > 0 && root.fraction < 1
                ? root.motionClock.phase + root.phaseOffset : root.phaseOffset
            property real available: root.hasValue ? 1 : 0
            fragmentShader: "../assets/shaders/liquid-gauge.frag.qsb"
        }
        ClippingRectangle {
            anchors.centerIn: parent
            visible: root.reducedEffects
            width: parent.width * 0.94; height: width
            radius: width / 2
            color: "#101d31"
            border.width: 1; border.color: "#5b9eb7d2"
            Rectangle {
                x: 0; y: parent.height * (1 - root.targetLevel)
                width: parent.width; height: parent.height * root.targetLevel
                visible: root.hasValue
                color: Qt.rgba(root.accent.r * 0.55, root.accent.g * 0.55, root.accent.b * 0.55, 1)
            }
        }
        Column {
            anchors.centerIn: parent
            spacing: 2
            visible: root.showLabels
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: root.label
                color: root.hasValue ? "#c5d6e9" : Theme.inkFaint
                font.family: Theme.fontSans; font.pixelSize: Theme.px(9)
                font.weight: Font.Medium; font.letterSpacing: 1
                style: Text.Outline; styleColor: "#a606101c"
            }
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: root.valueText
                color: root.hasValue ? Theme.ink : Theme.inkFaint
                font.family: Theme.fontMono; font.pixelSize: Theme.px(17)
                font.weight: Font.DemiBold
                style: Text.Outline; styleColor: "#a606101c"
            }
        }
    }
    Accessible.role: Accessible.StaticText
    Accessible.name: label
    Accessible.description: hasValue ? valueText : "Metric unavailable"
    Component.onCompleted: { displayedLevel = targetLevel; initialized = true; }
}
