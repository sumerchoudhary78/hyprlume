import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Time, date, and machine uptime. Sized to sit over the stage's top-right
// corner the way the reference does.
GlassSurface {
    id: root

    required property Clock clock
    required property Metrics metrics

    implicitWidth: Theme.px(186)
    implicitHeight: Theme.px(126)
    radius: Theme.radiusCard

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(16)
        spacing: Theme.px(2)

        Text {
            Layout.fillWidth: true
            text: root.clock.time
            color: Theme.ink
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontDisplay
            font.weight: Font.Light
            horizontalAlignment: Text.AlignRight
        }

        Text {
            Layout.fillWidth: true
            text: root.clock.date
            color: Theme.inkDim
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontCaption
            horizontalAlignment: Text.AlignRight
        }

        Item { Layout.fillHeight: true }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.divider
        }

        Text {
            Layout.fillWidth: true
            text: "UP " + root.metrics.formatUptime(root.metrics.uptimeMinutes)
                + "   LOAD " + root.metrics.load
            color: Theme.inkFaint
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontMicro
            horizontalAlignment: Text.AlignRight
        }
    }
}
