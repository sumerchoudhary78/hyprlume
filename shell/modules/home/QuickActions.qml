import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Five actions that exist today. The reference card lists deployment
// commands; those belong to a product that does not exist yet, so the same
// layout carries real shell operations instead.
GlassSurface {
    id: root

    required property Backend backend
    required property Workspaces workspaces

    radius: Theme.radiusCard

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.px(16)
        spacing: Theme.px(6)

        PanelHeader {
            Layout.fillWidth: true
            title: "Quick Actions"
            iconName: "flash-symbolic"
            accent: Theme.amber

            Chip {
                text: "Super K"
                textColor: Theme.inkFaint
                mono: true
            }
        }

        Item { Layout.preferredHeight: Theme.px(2) }

        ActionRow {
            Layout.fillWidth: true
            iconName: "utilities-terminal"
            label: "Run command"
            hint: "⏎"
            accent: Theme.amber
            onClicked: root.backend.send({ op: "toggle" })
        }

        ActionRow {
            Layout.fillWidth: true
            visible: root.height >= Theme.px(286)
            iconName: "go-next-symbolic"
            label: "Next workspace"
            hint: "e+1"
            onClicked: root.workspaces.focusNext()
        }

        ActionRow {
            Layout.fillWidth: true
            visible: root.height >= Theme.px(286)
            iconName: "go-previous-symbolic"
            label: "Previous workspace"
            hint: "e-1"
            onClicked: root.workspaces.focusPrevious()
        }

        ActionRow {
            Layout.fillWidth: true
            iconName: "audio-input-microphone-symbolic"
            label: root.backend.snapshot.voice === "wake" ? "Disable wake word" : "Enable wake word"
            active: root.backend.snapshot.voice === "wake"
            accent: Theme.mint
            onClicked: root.backend.send({
                op: "voice",
                mode: root.backend.snapshot.voice === "wake" ? "off" : "wake"
            })
        }

        ActionRow {
            Layout.fillWidth: true
            visible: root.height >= Theme.px(226)
            iconName: "media-record-symbolic"
            label: "Start dictation"
            active: root.backend.snapshot.voice === "dictate"
            accent: Theme.violet
            onClicked: root.backend.send({ op: "voice", mode: "dictate" })
        }

        Item { Layout.fillHeight: true }
    }
}
