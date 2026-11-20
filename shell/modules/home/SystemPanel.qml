import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var metrics
    readonly property bool expanded: detailsButton.active

    implicitHeight: content.implicitHeight + Theme.px(36)

    ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.px(18)
        spacing: Theme.px(12)

        PanelHeader {
            Layout.fillWidth: true
            title: "System"
            IconTile {
                id: detailsButton
                objectName: "systemDetails"
                size: Theme.px(32)
                iconName: root.expanded ? "minus" : "plus"
                tooltip: root.expanded ? "Hide system details" : "Show system details"
                quiet: true
                onClicked: active = !active
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(13)

            Repeater {
                model: ["CPU", "Memory", "GPU"]
                delegate: ColumnLayout {
                    id: metric
                    required property string modelData
                    required property int index
                    readonly property int value: index === 0 ? root.metrics.cpu : index === 1 ? root.metrics.mem
                        : root.metrics.gpuAvailable ? root.metrics.gpu : -1
                    Layout.fillWidth: true
                    Layout.preferredWidth: 1
                    spacing: Theme.px(4)

                    Text {
                        text: metric.modelData
                        color: Theme.inkDim
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.fontCaption
                    }

                    Text {
                        text: metric.value >= 0 ? metric.value + "%" : "—"
                        color: Theme.ink
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.px(22)
                        font.weight: Font.DemiBold
                        Accessible.name: metric.modelData + (metric.value >= 0 ? ", " + text : " unavailable")
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: Theme.px(3)
                        radius: height / 2
                        color: Theme.divider
                        Rectangle {
                            height: parent.height
                            width: parent.width * Math.max(0, Math.min(metric.value, 100)) / 100
                            radius: height / 2
                            color: Theme.cyan
                        }
                    }
                }
            }
        }

        Loader {
            objectName: "systemDetailsContent"
            visible: root.expanded
            active: root.expanded
            Layout.fillWidth: true

            sourceComponent: ColumnLayout {
                id: details
                readonly property var readings: [
                    "↑ " + root.metrics.formatRate(root.metrics.netUp) + "   ↓ " + root.metrics.formatRate(root.metrics.netDown),
                    root.metrics.formatBytes(root.metrics.diskUsed),
                    root.metrics.load,
                    root.metrics.formatUptime(root.metrics.uptimeMinutes),
                    root.metrics.containerRuntime
                        ? root.metrics.containerCount + " running · " + root.metrics.containerRuntime
                        : "Runtime unavailable"
                ]
                spacing: Theme.px(10)

                Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider }

                Repeater {
                    model: ["Network", "Disk used", "Load average", "Uptime", "Containers"]
                    delegate: RowLayout {
                        required property string modelData
                        required property int index
                        Layout.fillWidth: true
                        spacing: Theme.px(10)

                        Text {
                            text: modelData
                            color: Theme.inkDim
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.fontCaption
                        }
                        Text {
                            Layout.fillWidth: true
                            text: details.readings[index]
                            textFormat: Text.PlainText
                            horizontalAlignment: Text.AlignRight
                            color: Theme.ink
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.fontCaption
                            wrapMode: Text.Wrap
                        }
                    }
                }

                Text {
                    visible: !!root.metrics.containerName
                    Layout.fillWidth: true
                    text: root.metrics.containerName + " · " + root.metrics.containerCpu.toFixed(1) + "% CPU · " + root.metrics.containerMem
                    textFormat: Text.PlainText
                    color: Theme.inkDim
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontCaption
                    wrapMode: Text.Wrap
                }

                Sparkline {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.px(26)
                    visible: root.metrics.cpuHistory.length > 1
                    values: root.metrics.cpuHistory
                    accent: Theme.cyan
                }
                Text {
                    visible: root.metrics.cpuHistory.length > 1
                    text: "CPU history"
                    color: Theme.inkDim
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontCaption
                }
            }
        }
    }
}
