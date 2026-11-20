import QtQuick
import qs.theme

// Offset only the card. Inputs and the Wayland scrim are available immediately.
Translate {
    id: root
    property bool reducedEffects: false
    readonly property bool motionEnabled: !reducedEffects && !Theme.highContrast

    NumberAnimation on y {
        id: slide
        running: false
        from: Theme.px(8)
        to: 0
        duration: Theme.motionBase
        easing.type: Theme.easing
    }

    Component.onCompleted: if (motionEnabled) {
        y = slide.from;
        slide.start();
    }

    onMotionEnabledChanged: if (!motionEnabled && slide.running) {
        slide.stop();
        y = 0;
    }
}
