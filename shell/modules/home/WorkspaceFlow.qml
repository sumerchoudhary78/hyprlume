import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Workspace flow: the live workspace list as a connected spine. It replaces
// the reference's build/test/deploy graph, which has no data behind it.
GlassSurface {
    id: root

    required property Workspaces workspaces

    radius: Theme.radiusCard

    readonly property int maxRows: Math.max(1, Math.min(6,
        Math.floor((height - Theme.px(86)) / Theme.px(44))))

    readonly property var nodes: {
        const list = (root.workspaces.all ?? []).slice();
        list.sort((a, b) => a.id - b.id);
        return list.slice(0, root.maxRows);
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(16)
        spacing: Theme.px(10)

        PanelHeader {
            Layout.fillWidth: true
            title: "Workspace Flow"
            iconName: "view-list-details"
            accent: Theme.violet

            Text {
                text: root.nodes.length + " open"
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontCaption
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Spine behind the nodes.
            Rectangle {
                x: Theme.px(11)
                y: Theme.px(20)
                width: 1
                height: Math.max(0, parent.height - Theme.px(46))
                gradient: Gradient {
                    GradientStop { position: 0.0; color: Qt.rgba(0.64, 0.55, 1.0, 0.45) }
                    GradientStop { position: 1.0; color: Qt.rgba(0.64, 0.55, 1.0, 0.04) }
                }
            }

            Column {
                anchors.fill: parent
                spacing: 0

                Repeater {
                    model: root.nodes

                    delegate: Item {
                        id: node

                        required property var modelData

                        width: parent.width
                        height: Theme.px(44)

                        readonly property bool current: node.modelData.id === root.workspaces.focusedId

                        Rectangle {
                            anchors.verticalCenter: parent.verticalCenter
                            x: Theme.px(5)
                            width: Theme.px(13)
                            height: width
                            radius: width / 2
                            color: node.current ? Theme.amber : Theme.violet
                            opacity: node.current ? 1 : 0.45

                            Rectangle {
                                anchors.centerIn: parent
                                visible: node.current
                                width: parent.width + Theme.px(8)
                                height: width
                                radius: width / 2
                                color: Theme.amber
                                opacity: 0.18
                            }
                        }

                        Rectangle {
                            anchors.verticalCenter: parent.verticalCenter
                            x: Theme.px(26)
                            width: parent.width - Theme.px(26)
                            height: Theme.px(34)
                            radius: Theme.radiusTile
                            color: nodeMouse.containsMouse || node.current
                                ? Qt.rgba(1, 1, 1, 0.06)
                                : "transparent"
                            border.width: node.current ? 1 : 0
                            border.color: Qt.rgba(0.95, 0.65, 0.35, 0.32)

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Theme.px(10)
                                anchors.rightMargin: Theme.px(10)
                                spacing: Theme.px(6)

                                Text {
                                    Layout.fillWidth: true
                                    text: root.workspaces.label(node.modelData)
                                    color: node.current ? Theme.ink : Theme.inkDim
                                    font.family: Theme.fontSans
                                    font.pixelSize: Theme.fontBody
                                    elide: Text.ElideRight
                                }

                                Text {
                                    text: root.workspaces.countFor(node.modelData)
                                    color: Theme.inkFaint
                                    font.family: Theme.fontMono
                                    font.pixelSize: Theme.fontCaption
                                }
                            }

                            MouseArea {
                                id: nodeMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.workspaces.focus(node.modelData)
                            }
                        }
                    }
                }
            }

            Text {
                visible: root.nodes.length === 0
                anchors.centerIn: parent
                text: "No workspaces reported"
                color: Theme.inkFaint
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontCaption
            }
        }
    }
}
