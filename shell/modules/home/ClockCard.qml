import QtQuick
import qs.components
import qs.theme

Item {
    id: root
    required property var clock
    required property var metrics
    implicitWidth: 165
    implicitHeight: 145
    Canvas {
        anchors.fill: parent
        onPaint: {
            const c = getContext("2d");c.clearRect(0,0,width,height);
            c.beginPath();c.moveTo(30,1);c.lineTo(77,1);c.moveTo(15,16);c.lineTo(1,31);
            c.lineTo(1,height-22);c.lineTo(13,height-10);c.lineTo(83,height-10);c.lineTo(94,height-1);c.lineTo(width,height-1);
            c.strokeStyle="#7cccf4";c.lineWidth=0.8;c.stroke();
        }
    }
    Text { anchors.right: parent.right; y: 7; text: root.clock.time; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 48; font.weight: Font.ExtraLight }
    Text { anchors.right: parent.right; y: 67; text: root.clock.date; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 14 }
    Rectangle { x: 25; y: 96; width: parent.width - 25; height: 1; color: Theme.divider }
    Row {
        anchors.right: parent.right; y: 108; spacing: 8
        Icon { name: "cpu"; size: 15; color: Theme.inkDim }
        Text { text: "Up " + root.metrics.formatUptime(root.metrics.uptimeMinutes); color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 12 }
    }
}
