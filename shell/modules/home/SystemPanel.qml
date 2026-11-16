import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// CPU, memory, GPU, network, and a short CPU trend.
GlassSurface {
    id: root

    required property Metrics metrics

    radius: Theme.radiusCard

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(14)
        spacing: Theme.px(8)

        PanelHeader {
            Layout.fillWidth: true
            title: "System"
            iconName: "utilities-system-monitor"
            accent: Theme.mint

            Text {
                text: "load " + root.metrics.load
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontMicro
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Theme.px(6)

            Gauge {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(74)
                arcSize: Theme.px(74)
                thickness: Theme.px(5)
                value: root.metrics.cpu / 100
                accent: Theme.cyan
                label: "CPU"
            }

            Gauge {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(74)
                arcSize: Theme.px(74)
                thickness: Theme.px(5)
                value: root.metrics.mem / 100
                accent: Theme.violet
                label: "RAM"
                valueText: root.metrics.memMb + "M"
            }

            Gauge {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(74)
                arcSize: Theme.px(74)
                thickness: Theme.px(5)
                value: root.metrics.gpuAvailable ? root.metrics.gpu / 100 : 0
                accent: Theme.mint
                label: "GPU"
                valueText: root.metrics.gpuAvailable
                    ? root.metrics.gpu + "%"
                    : "—"
            }
        }

        Sparkline {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.px(34)
            values: root.metrics.cpuHistory
            accent: Theme.cyan
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(12)

            Row {
                spacing: Theme.px(4)

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "go-up-symbolic"
                    size: Theme.px(11)
                    color: Theme.mint
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.metrics.formatRate(root.metrics.netUp)
                    color: Theme.inkDim
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.fontMicro
                }
            }

            Row {
                spacing: Theme.px(4)

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "go-down-symbolic"
                    size: Theme.px(11)
                    color: Theme.cyan
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.metrics.formatRate(root.metrics.netDown)
                    color: Theme.inkDim
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.fontMicro
                }
            }

            Item { Layout.fillWidth: true }

            Text {
                text: root.metrics.formatBytes(root.metrics.diskUsed)
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontMicro
            }
        }
    }
}
