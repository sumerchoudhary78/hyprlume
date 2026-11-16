import QtQuick

// One bounded clock shared by the three water gauges. A reading or hover
// wakes it; settled, hidden and reduced-motion surfaces have no frame timer.
Item {
    id: root
    property bool active: true
    property bool reducedMotion: false
    property bool hovered: false
    property real phase: 1.2
    property int settleDuration: 3200
    property int tickCount: 0
    readonly property bool running: frames.running
    signal advanced(real seconds)

    function pulse() {
        if (active && !reducedMotion) settling.restart();
    }
    onActiveChanged: {
        if (active) pulse(); else settling.stop();
    }
    onReducedMotionChanged: {
        if (reducedMotion) settling.stop(); else pulse();
    }
    Timer {
        id: settling
        interval: Math.max(0, root.settleDuration)
        repeat: false
    }
    Timer {
        id: frames
        // At this small amplitude, 16 Hz moves the wave by subpixel steps.
        interval: 62
        repeat: true
        running: root.active && !root.reducedMotion && (root.hovered || settling.running)
        onTriggered: {
            root.phase = (root.phase + interval / 1000) % 4096;
            root.tickCount++;
            root.advanced(interval / 1000);
        }
    }
    Component.onCompleted: pulse()
}
