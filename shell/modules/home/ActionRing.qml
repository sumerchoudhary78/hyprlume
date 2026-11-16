import QtQuick
import QtQuick.Shapes
import qs.components
import qs.services
import qs.theme

// Radial launcher for the shell's real verbs. The centre opens the command
// panel; three satellites switch voice modes and the fourth changes workspace.
Item {
    id: root

    required property Backend backend
    required property Workspaces workspaces

    implicitHeight: Theme.px(196)

    readonly property string voice: backend.snapshot.voice ?? "off"
    readonly property real radiusX: Math.min(width * 0.34, Theme.px(150))
    readonly property real radiusY: Math.min(height * 0.30, Theme.px(62))

    readonly property var actions: [
        { label: "Wake", icon: "audio-input-microphone-symbolic", mode: "wake" },
        { label: "Dictate", icon: "media-record-symbolic", mode: "dictate" },
        { label: "Mic off", icon: "microphone-sensitivity-muted-symbolic", mode: "off" },
        { label: "Next ws", icon: "go-next-symbolic", mode: "" }
    ]

    function angleFor(index) {
        return (-90 + index * 90) * Math.PI / 180;
    }

    // -- Orbits --------------------------------------------------------------
    Shape {
        anchors.fill: parent
        antialiasing: true
        opacity: 0.5

        ShapePath {
            fillColor: "transparent"
            strokeColor: Qt.rgba(0.37, 0.85, 1.0, 0.16)
            strokeWidth: 1
            PathAngleArc {
                centerX: root.width / 2
                centerY: root.height / 2
                radiusX: root.radiusX + Theme.px(18)
                radiusY: root.radiusY + Theme.px(12)
                startAngle: 0
                sweepAngle: 360
            }
        }

        ShapePath {
            fillColor: "transparent"
            strokeColor: Qt.rgba(0.95, 0.65, 0.35, 0.30)
            strokeWidth: Math.max(1, Theme.px(2))
            capStyle: ShapePath.RoundCap
            PathAngleArc {
                centerX: root.width / 2
                centerY: root.height / 2
                radiusX: root.radiusX + Theme.px(26)
                radiusY: root.radiusY + Theme.px(18)
                startAngle: 200
                sweepAngle: 70
            }
        }
    }

    // -- Satellites ----------------------------------------------------------
    Repeater {
        model: root.actions

        delegate: Item {
            id: satellite

            required property var modelData
            required property int index

            readonly property bool active: (satellite.modelData.mode === "wake"
                || satellite.modelData.mode === "dictate")
                && root.voice === satellite.modelData.mode

            width: Theme.px(74)
            height: Theme.px(54)
            x: root.width / 2 + Math.cos(root.angleFor(satellite.index)) * root.radiusX - width / 2
            y: root.height / 2 + Math.sin(root.angleFor(satellite.index)) * root.radiusY - height / 2

            Rectangle {
                anchors.fill: parent
                radius: Theme.radiusTile
                color: satellite.active
                    ? Qt.rgba(0.45, 0.90, 0.72, 0.85)
                    : (mouse.containsMouse ? Theme.hover : Qt.rgba(0.03, 0.07, 0.12, 0.72))
                border.width: 1
                border.color: satellite.active
                    ? Qt.rgba(0.55, 0.95, 0.78, 0.9)
                    : Theme.rim
            }

            Column {
                anchors.centerIn: parent
                spacing: Theme.px(3)

                Icon {
                    anchors.horizontalCenter: parent.horizontalCenter
                    name: satellite.modelData.icon
                    size: Theme.px(16)
                    color: satellite.active ? "#0d1a16" : Theme.inkDim
                }

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: satellite.modelData.label
                    color: satellite.active ? "#0d1a16" : Theme.inkFaint
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontMicro
                    font.weight: Font.Medium
                }
            }

            MouseArea {
                id: mouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    if (satellite.modelData.mode.length > 0)
                        root.backend.send({ op: "voice", mode: satellite.modelData.mode });
                    else
                        root.workspaces.focusNext();
                }
            }
        }
    }

    // -- Core ----------------------------------------------------------------
    Item {
        id: core

        anchors.centerIn: parent
        width: Theme.px(82)
        height: width

        Rectangle {
            anchors.centerIn: parent
            width: parent.width + Theme.px(18)
            height: width
            radius: width / 2
            color: Qt.rgba(0.37, 0.85, 1.0, coreMouse.containsMouse ? 0.18 : 0.10)
        }

        Rectangle {
            anchors.fill: parent
            radius: width / 2
            gradient: Gradient {
                GradientStop { position: 0.0; color: Qt.rgba(0.30, 0.62, 0.86, 0.95) }
                GradientStop { position: 0.55; color: Qt.rgba(0.16, 0.34, 0.58, 0.95) }
                GradientStop { position: 1.0; color: Qt.rgba(0.07, 0.15, 0.27, 0.98) }
            }
            border.width: 1
            border.color: Qt.rgba(0.55, 0.90, 1.0, 0.55)
        }

        Icon {
            anchors.centerIn: parent
            name: "utilities-terminal"
            size: Theme.px(26)
            color: Theme.ink
        }

        MouseArea {
            id: coreMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.backend.send({ op: "toggle" })
        }
    }

    Text {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        text: root.backend.snapshot.panel_open ? "Close command panel" : "Open command panel"
        color: Theme.inkFaint
        font.family: Theme.fontSans
        font.pixelSize: Theme.fontMicro
    }
}
