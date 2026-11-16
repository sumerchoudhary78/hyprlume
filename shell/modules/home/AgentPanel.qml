import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Assistant panel: identity orb, the daemon's latest message, prompt
// shortcuts, and the microphone state.
GlassSurface {
    id: root

    required property Backend backend
    property bool focused: false

    radius: Theme.radiusCard
    glowing: root.focused
    glowColor: Theme.violet

    readonly property string voice: backend.snapshot.voice ?? "off"
    readonly property bool showShortcuts: height >= Theme.px(380)

    function ask(text) {
        backend.send({ op: "command", text: text, dry_run: true });
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(16)
        spacing: Theme.px(10)

        PanelHeader {
            Layout.fillWidth: true
            title: "Shell Agent"
            iconName: "applications-science"
            accent: Theme.violet

            StatusDot {
                Layout.alignment: Qt.AlignVCenter
                size: Theme.px(7)
                color: root.backend.online ? Theme.mint : Theme.rose
            }

            Text {
                Layout.alignment: Qt.AlignVCenter
                text: root.backend.online ? "Online" : "Offline"
                color: Theme.inkDim
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontCaption
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(12)

            AgentOrb {
                Layout.alignment: Qt.AlignTop
                active: root.voice !== "off"
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: messageLabel.implicitHeight + Theme.px(20)
                radius: Theme.radiusTile
                color: Qt.rgba(1, 1, 1, 0.045)
                border.width: 1
                border.color: Theme.divider

                Text {
                    id: messageLabel
                    anchors.fill: parent
                    anchors.margins: Theme.px(10)
                    text: root.backend.snapshot.message
                    color: Theme.ink
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontBody
                    wrapMode: Text.Wrap
                    verticalAlignment: Text.AlignVCenter
                    maximumLineCount: 3
                    elide: Text.ElideRight
                }
            }
        }

        ActionRow {
            Layout.fillWidth: true
            visible: root.showShortcuts
            iconName: "utilities-terminal"
            label: "Analyze this codebase"
            onClicked: root.ask("analyze this codebase and summarize the architecture")
        }

        ActionRow {
            Layout.fillWidth: true
            visible: root.showShortcuts
            iconName: "dialog-warning-symbolic"
            label: "Find issues and fix them"
            onClicked: root.ask("find issues in this project and propose fixes")
        }

        ActionRow {
            Layout.fillWidth: true
            visible: root.showShortcuts
            iconName: "go-up-symbolic"
            label: "Create a deployment flow"
            onClicked: root.ask("create a deployment flow for this project")
        }

        Item { Layout.fillHeight: true }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.divider
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(10)

            Item {
                Layout.fillWidth: true
                implicitHeight: Theme.px(24)

                WaveBars {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(parent.width, Theme.px(150))
                    active: root.voice !== "off"
                    accent: root.voice === "dictate" ? Theme.violet : Theme.mint
                }
            }

            Text {
                text: root.voice === "off" ? "MICROPHONE OFF"
                    : root.voice === "wake" ? "WAKE WORD"
                    : "DICTATING"
                color: Theme.inkFaint
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontMicro
            }

            IconTile {
                iconName: root.voice === "off" ? "audio-input-microphone-symbolic" : "audio-volume-high-symbolic"
                active: root.voice !== "off"
                accent: Theme.mint
                size: Theme.px(34)
                iconSize: Theme.px(15)
                onClicked: root.backend.send({
                    op: "voice",
                    mode: root.voice === "off" ? "wake" : "off"
                })
            }
        }
    }

    component AgentOrb: Item {
        id: orb

        property bool active: false

        implicitWidth: Theme.px(54)
        implicitHeight: Theme.px(54)

        Rectangle {
            anchors.centerIn: parent
            width: parent.width + Theme.px(10)
            height: width
            radius: width / 2
            color: Qt.rgba(0.64, 0.55, 1.0, orb.active ? 0.26 : 0.12)
        }

        Rectangle {
            anchors.fill: parent
            radius: width / 2
            gradient: Gradient {
                GradientStop { position: 0.0; color: Qt.rgba(0.55, 0.62, 0.92, 1.0) }
                GradientStop { position: 0.55; color: Qt.rgba(0.22, 0.26, 0.48, 1.0) }
                GradientStop { position: 1.0; color: Qt.rgba(0.06, 0.08, 0.16, 1.0) }
            }
            border.width: 1
            border.color: Qt.rgba(0.70, 0.72, 1.0, 0.45)
        }

        // Face plate.
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 0.62
            height: width * 0.78
            radius: width / 2
            color: Qt.rgba(0.04, 0.07, 0.13, 0.9)
            border.width: 1
            border.color: Qt.rgba(0.55, 0.90, 1.0, 0.35)
        }

        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 0.5
            height: Theme.px(2)
            radius: height / 2
            color: orb.active ? Theme.cyan : Qt.rgba(0.55, 0.90, 1.0, 0.5)
        }

        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.leftMargin: Theme.px(9)
            anchors.topMargin: Theme.px(8)
            width: Theme.px(12)
            height: Theme.px(7)
            radius: width / 2
            color: Qt.rgba(1, 1, 1, 0.35)
        }
    }
}
