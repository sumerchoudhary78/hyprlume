import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// CPU, memory, GPU, network, and a short CPU trend.
GlassSurface {
    id: root

    required property var metrics
    property bool animationsActive: true
    readonly property bool liquidMotionRunning: waterMotion.running
    readonly property int liquidTicks: waterMotion.tickCount
    objectName: "systemMetrics"

    LiquidMotion {
        id: waterMotion
        active: root.animationsActive && root.visible
        reducedMotion: root.reducedEffects
        hovered: waterHover.hovered
    }
    HoverHandler { id: waterHover }
    Connections {
        target: root.metrics
        function onCpuChanged() { waterMotion.pulse(); }
        function onMemChanged() { waterMotion.pulse(); }
        function onGpuChanged() { waterMotion.pulse(); }
    }

    radius: Theme.radiusCard

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(14)
        spacing: Theme.px(8)

        PanelHeader {
            Layout.fillWidth: true
            title: "System"
            iconName: "utilities-system-monitor"
            accent: Theme.amber

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
                Layout.preferredHeight: Theme.px(86)
                diameter: Theme.px(86)
                motionClock: waterMotion
                reducedEffects: root.reducedEffects
                value: root.metrics.cpu / 100
                accent: Theme.mint
                label: "CPU"
                phaseOffset: 0.0
                objectName: "cpuGauge"
            }

            Gauge {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(86)
                diameter: Theme.px(86)
                motionClock: waterMotion
                reducedEffects: root.reducedEffects
                value: root.metrics.mem / 100
                accent: Theme.cyan
                label: "RAM"
                phaseOffset: 1.8
                objectName: "memoryGauge"
                valueText: root.metrics.mem + "%"
            }

            Gauge {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(86)
                diameter: Theme.px(86)
                motionClock: waterMotion
                reducedEffects: root.reducedEffects
                value: root.metrics.gpuAvailable ? root.metrics.gpu / 100 : 0
                accent: Theme.violet
                label: "GPU"
                available: root.metrics.gpuAvailable
                phaseOffset: 3.5
                objectName: "gpuGauge"
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
