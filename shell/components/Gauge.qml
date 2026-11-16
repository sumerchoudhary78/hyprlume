import QtQuick
import QtQuick.Shapes
import qs.theme

// Circular gauge: a 260-degree arc over a faint track, with the value in the
// middle. Used by the system card for CPU, memory, and GPU.
Item {
    id: root

    property real value: 0
    property color accent: Theme.cyan
    property string label: ""
    property string valueText: Math.round(Math.max(0, Math.min(1, value)) * 100) + "%"
    property int thickness: Theme.px(6)
    property int arcSize: Theme.px(88)

    implicitWidth: arcSize
    implicitHeight: arcSize

    readonly property real clamped: Math.max(0, Math.min(1, value))
    readonly property real radiusX: (Math.min(width, height) - thickness) / 2

    Shape {
        anchors.fill: parent
        antialiasing: true

        ShapePath {
            fillColor: "transparent"
            strokeColor: Theme.divider
            strokeWidth: root.thickness
            capStyle: ShapePath.RoundCap
            PathAngleArc {
                centerX: root.width / 2
                centerY: root.height / 2
                radiusX: root.radiusX
                radiusY: root.radiusX
                startAngle: 140
                sweepAngle: 260
            }
        }

        ShapePath {
            fillColor: "transparent"
            strokeColor: root.accent
            strokeWidth: root.thickness
            capStyle: ShapePath.RoundCap
            PathAngleArc {
                centerX: root.width / 2
                centerY: root.height / 2
                radiusX: root.radiusX
                radiusY: root.radiusX
                startAngle: 140
                sweepAngle: 260 * root.clamped
            }
        }
    }

    Column {
        anchors.centerIn: parent
        spacing: Theme.px(1)

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.valueText
            color: Theme.ink
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontLabel
            font.weight: Font.DemiBold
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.label
            color: Theme.inkFaint
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontMicro
            font.weight: Font.Medium
        }
    }
}
