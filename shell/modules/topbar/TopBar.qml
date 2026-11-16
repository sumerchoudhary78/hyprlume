import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import qs.components
import qs.services
import qs.theme

// The always-on bar. It owns a reserved strip at the top of the screen so
// Hyprland tiles windows below it instead of underneath it.
//
// Blur is requested by the compositor layer rule in docs/setup.md; this
// surface only paints the glass tint.
ShellWindow {
    id: root
    surfaceName: "bar"

    required property Backend backend
    required property Workspaces workspaces
    required property ShellState state

    readonly property bool showModeLabels: width >= Theme.px(1380)
    readonly property int visibleModes: {
        if (width >= Theme.px(1560))
            return 6;
        if (width >= Theme.px(1240))
            return 5;
        if (width >= Theme.px(980))
            return 3;
        return 0;
    }

    readonly property var modes: [
        { label: "Terminal", icon: "utilities-terminal" },
        { label: "Workflows", icon: "go-next-symbolic" },
        { label: "Agents", icon: "applications-science" },
        { label: "Scripts", icon: "text-x-script" },
        { label: "Containers", icon: "package-x-generic" },
        { label: "Cloud", icon: "weather-overcast-symbolic" }
    ]

    anchors {
        top: true
        left: true
        right: true
    }
    margins {
        top: Theme.px(12)
        left: Theme.px(14)
        right: Theme.px(14)
    }
    implicitHeight: Theme.barHeight
    exclusiveZone: Theme.barZone

    WlrLayershell.layer: WlrLayer.Top

    GlassSurface {
        anchors.fill: parent
        radius: Theme.radiusCard

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.px(16)
            anchors.rightMargin: Theme.px(12)
            spacing: Theme.px(10)

            Brand {
                compact: root.width < Theme.px(820)
                Layout.alignment: Qt.AlignVCenter
            }

            Rectangle {
                visible: root.visibleModes > 0
                Layout.preferredWidth: 1
                Layout.preferredHeight: Theme.px(22)
                Layout.leftMargin: Theme.px(6)
                color: Theme.divider
            }

            Row {
                visible: root.visibleModes > 0
                Layout.alignment: Qt.AlignVCenter
                spacing: Theme.px(2)

                Repeater {
                    model: root.modes

                    ModeTab {
                        required property int index
                        required property var modelData
                        visible: index < root.visibleModes
                        label: modelData.label
                        iconName: modelData.icon
                        showLabel: root.showModeLabels
                        active: root.state.mode === index
                        onClicked: root.state.mode = index
                    }
                }
            }

            Item { Layout.fillWidth: true }

            SearchBox {
                visible: root.width >= Theme.px(900)
                Layout.preferredWidth: Math.min(Theme.px(320), root.width * 0.26)
                Layout.alignment: Qt.AlignVCenter
                onActivated: root.backend.send({ op: "toggle" })
            }

            MicChip {
                mode: root.backend.snapshot.voice ?? "off"
                online: root.backend.online
                onToggled: root.backend.send({
                    op: "voice",
                    mode: root.backend.snapshot.voice === "off" ? "wake" : "off"
                })
            }

            AvatarOrb {
                online: root.backend.online
                onClicked: root.backend.send({ op: "toggle" })
            }
        }
    }

    // -- Small parts ---------------------------------------------------------

    component MicChip: Item {
        id: chip

        required property string mode
        required property bool online

        signal toggled()

        implicitWidth: chipRow.implicitWidth + Theme.px(22)
        implicitHeight: Theme.px(38)

        Rectangle {
            anchors.fill: parent
            radius: Theme.px(12)
            color: chipMouse.containsMouse ? Theme.hover : "transparent"
            border.width: 1
            border.color: Theme.divider
        }

        Row {
            id: chipRow
            anchors.centerIn: parent
            spacing: Theme.px(8)

            StatusDot {
                id: chipModeDot
                anchors.verticalCenter: parent.verticalCenter
                size: Theme.px(7)
                color: !chip.online ? Theme.rose
                    : chip.mode === "off" ? Theme.inkFaint
                    : chip.mode === "wake" ? Theme.mint
                    : Theme.violet
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: !chip.online ? "Daemon offline"
                    : chip.mode === "off" ? "Mic off"
                    : chip.mode === "wake" ? "Wake word"
                    : "Dictating"
                color: Theme.inkDim
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontCaption
                font.weight: Font.Medium
            }
        }

        MouseArea {
            id: chipMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: chip.toggled()
        }
    }

    component AvatarOrb: Item {
        id: orb

        required property bool online

        signal clicked()

        implicitWidth: Theme.px(34)
        implicitHeight: Theme.px(34)

        Rectangle {
            anchors.centerIn: parent
            width: parent.width + Theme.px(6)
            height: width
            radius: width / 2
            color: Qt.rgba(0.55, 0.85, 1.0, orbMouse.containsMouse ? 0.22 : 0.10)
        }

        Rectangle {
            anchors.fill: parent
            radius: width / 2
            gradient: Gradient {
                GradientStop { position: 0.0; color: Qt.rgba(0.34, 0.55, 0.72, 1.0) }
                GradientStop { position: 1.0; color: Qt.rgba(0.10, 0.18, 0.28, 1.0) }
            }
            border.width: 1
            border.color: Qt.rgba(0.62, 0.88, 1.0, 0.45)
        }

        Rectangle {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            width: Theme.px(9)
            height: width
            radius: width / 2
            color: orb.online ? Theme.mint : Theme.rose
            border.width: 2
            border.color: "#0b131c"
        }

        MouseArea {
            id: orbMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: orb.clicked()
        }
    }
}
