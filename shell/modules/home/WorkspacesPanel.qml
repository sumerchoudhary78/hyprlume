import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Live Hyprland workspaces. Clicking a row switches to it.
GlassSurface {
    id: root

    required property Workspaces workspaces

    radius: Theme.radiusCard

    readonly property int maxRows: Math.max(1, Math.min(6,
        Math.floor((height - Theme.px(64)) / Theme.px(38))))

    readonly property var rows: {
        const list = (root.workspaces.all ?? []).slice();
        list.sort((a, b) => a.id - b.id);
        return list.slice(0, root.maxRows);
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(16)
        spacing: Theme.px(6)

        PanelHeader {
            Layout.fillWidth: true
            title: "Workspaces"
            iconName: "view-grid-symbolic"
            accent: Theme.cyan

            Text {
                text: root.rows.length + " open"
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontCaption
            }
        }

        Item { Layout.preferredHeight: Theme.px(2) }

        Repeater {
            model: root.rows

            delegate: Item {
                id: row

                required property var modelData

                Layout.fillWidth: true
                implicitHeight: Theme.px(38)

                readonly property bool current: row.modelData.id === root.workspaces.focusedId

                Rectangle {
                    anchors.fill: parent
                    radius: Theme.radiusTile
                    color: rowMouse.containsMouse || row.current ? Qt.rgba(1, 1, 1, 0.06) : "transparent"
                    border.width: row.current ? 1 : 0
                    border.color: Qt.rgba(0.37, 0.85, 1.0, 0.28)
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.px(10)
                    anchors.rightMargin: Theme.px(10)
                    spacing: Theme.px(9)

                    StatusDot {
                        size: Theme.px(7)
                        halo: row.current
                        color: row.current ? Theme.cyan : Theme.inkFaint
                    }

                    Text {
                        Layout.fillWidth: true
                        text: root.workspaces.label(row.modelData)
                        color: row.current ? Theme.ink : Theme.inkDim
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.fontBody
                        elide: Text.ElideRight
                    }

                    Text {
                        text: root.workspaces.countFor(row.modelData)
                        color: Theme.inkFaint
                        font.family: Theme.fontMono
                        font.pixelSize: Theme.fontCaption
                    }

                    Icon {
                        name: "go-next-symbolic"
                        size: Theme.px(13)
                        color: Theme.inkFaint
                        iconOpacity: rowMouse.containsMouse ? 1 : 0.4
                    }
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.workspaces.focus(row.modelData)
                }
            }
        }

        Text {
            visible: root.rows.length === 0
            Layout.fillWidth: true
            text: "Hyprland reports no workspaces"
            color: Theme.inkFaint
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontCaption
            wrapMode: Text.Wrap
        }

        Item { Layout.fillHeight: true }
    }
}
