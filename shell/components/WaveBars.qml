import QtQuick
import qs.theme

// Voice-state illustration, not a measured microphone amplitude.
// Only animate while the voice control is visible and enabled.
Item {
    id: root

    property bool active: false
    property bool animated: true
    property color accent: Theme.violet
    property int barCount: 26
    property int barWidth: 2
    property int barSpacing: 3
    property real amplitude: 0.8

    implicitWidth: barCount * (barWidth + barSpacing)
    implicitHeight: Theme.px(22)

    property real phase: 0

    NumberAnimation on phase {
        running: root.active && root.visible && root.animated
        loops: Animation.Infinite
        from: 0
        to: Math.PI * 2
        duration: 1500
        easing.type: Easing.Linear
    }

    Row {
        anchors.centerIn: parent
        spacing: root.barSpacing

        Repeater {
            model: root.barCount

            Rectangle {
                required property int index
                width: root.barWidth
                radius: width / 2
                anchors.verticalCenter: parent.verticalCenter
                color: root.accent
                opacity: root.active ? 0.9 : 0.5
                height: {
                    if (!root.active)
                        return Math.max(2, root.implicitHeight * Math.sin(index / root.barCount * Math.PI) * (0.12 + 0.6 * Math.pow(Math.sin(index * 2.1), 2)));
                    const wave = Math.sin(root.phase + index * 0.55);
                    const shape = Math.sin(index / root.barCount * Math.PI);
                    return Math.max(Theme.px(3), root.amplitude * root.implicitHeight * shape * (0.35 + 0.65 * wave * wave));
                }
                Behavior on height {
                    NumberAnimation { duration: Theme.motionFast }
                }
            }
        }
    }
}
