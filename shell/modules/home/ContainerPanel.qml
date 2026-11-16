import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Container state from docker or podman, whichever the sampler found.
// With neither installed the card says so instead of showing fake runtime
// numbers.
GlassSurface {
    id: root

    required property Metrics metrics

    radius: Theme.radiusCard

    readonly property bool hasRuntime: metrics.containerRuntime.length > 0
    readonly property bool running: metrics.containerCount > 0

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(16)
        spacing: Theme.px(8)

        PanelHeader {
            Layout.fillWidth: true
            title: "Containers"
            iconName: "package-x-generic"
            accent: Theme.cyan

            StatusDot {
                Layout.alignment: Qt.AlignVCenter
                size: Theme.px(7)
                halo: root.running
                color: root.running ? Theme.mint : Theme.inkFaint
            }

            Text {
                Layout.alignment: Qt.AlignVCenter
                text: !root.hasRuntime ? "No runtime"
                    : root.running ? "Running" : "Idle"
                color: root.running ? Theme.mint : Theme.inkDim
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontCaption
            }
        }

        Text {
            Layout.fillWidth: true
            visible: root.hasRuntime
            text: root.running
                ? root.metrics.containerName
                : "No containers running"
            color: root.running ? Theme.ink : Theme.inkFaint
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontTitle
            font.weight: Font.Medium
            elide: Text.ElideRight
        }

        RowLayout {
            Layout.fillWidth: true
            visible: root.running
            spacing: Theme.px(8)

            Chip {
                text: Math.round(root.metrics.containerCpu) + "% cpu"
                textColor: Theme.inkDim
            }

            Chip {
                text: (root.metrics.containerMem.length > 0 ? root.metrics.containerMem : "—") + " mem"
                textColor: Theme.inkDim
            }

            Item { Layout.fillWidth: true }

            Text {
                text: root.metrics.containerCount + " total"
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontMicro
            }
        }

        Item { Layout.fillHeight: true }

        Text {
            Layout.fillWidth: true
            visible: !root.hasRuntime
            text: "Install docker or podman to populate this card."
            color: Theme.inkFaint
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontCaption
            wrapMode: Text.Wrap
        }

        RowLayout {
            Layout.fillWidth: true
            visible: root.hasRuntime

            Text {
                text: "runtime  " + root.metrics.containerRuntime
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontMicro
            }

            Item { Layout.fillWidth: true }

            Text {
                text: "disk  " + root.metrics.formatBytes(root.metrics.diskUsed)
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontMicro
            }
        }
    }
}
