import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.models
import qs.theme

ColumnLayout {
    id: root
    required property var backend
    readonly property bool available: backend.online && !state.blocked && !backend.locking.active
    spacing: Theme.px(6)
    CommandState { id: state; snapshot: root.backend.snapshot }

    function preview(text) {
        if (available && text.trim())
            backend.send({op: "command", text: text.trim(), dry_run: true});
    }

    Text {
        Layout.fillWidth: true
        text: !root.backend.online ? "Command service offline" : state.busy ? "Command in progress" : "Command"
        color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: Theme.fontLabel; font.weight: Font.DemiBold
        Accessible.role: Accessible.Heading
    }
    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Theme.px(42)
        radius: Theme.px(9)
        color: Theme.field
        border.width: input.activeFocus ? 2 : 1
        border.color: input.activeFocus ? Theme.focusRing : Theme.rim

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.px(12); anchors.rightMargin: Theme.px(4)
            spacing: Theme.px(6)
            TextField {
                id: input
                objectName: "consoleInput"
                Layout.fillWidth: true; Layout.minimumWidth: 0
                padding: 0; background: null
                placeholderText: "Type a command…"
                font.family: Theme.fontSans; font.pixelSize: Theme.fontBody
                color: Theme.ink; placeholderTextColor: Theme.inkFaint
                selectionColor: Theme.cyan; selectedTextColor: Theme.onAccent
                enabled: root.available
                selectByMouse: true; maximumLength: 512
                Accessible.name: "Desktop command"
                Accessible.description: "Press Enter to preview"
                onAccepted: if (text.trim()) { root.preview(text); clear(); }
            }
            GlassButton {
                objectName: "consolePreview"
                compact: true; quiet: true
                text: "Preview"
                enabled: root.available && !!input.text.trim()
                onClicked: input.accepted()
            }
        }
    }
    Flow {
        Layout.fillWidth: true
        spacing: Theme.px(5)
        GlassButton {
            objectName: "launcherterminal"
            compact: true; quiet: true; iconName: "utilities-terminal"; text: "Open terminal"
            enabled: root.available
            onClicked: root.preview("launch terminal")
        }
        GlassButton {
            objectName: "dictateNote"
            compact: true; quiet: true; iconName: "audio-input-microphone"; text: "Dictate"
            enabled: root.backend.online && !root.backend.locking.active
            onClicked: root.backend.send({op: "voice", mode: "dictate"})
        }
        GlassButton {
            visible: state.review
            text: "Review plan"; compact: true
            onClicked: root.backend.send({op: "toggle"})
        }
    }
    Text {
        Layout.fillWidth: true
        visible: root.backend.online && !!root.backend.error
        text: root.backend.error
        textFormat: Text.PlainText; wrapMode: Text.Wrap
        color: Theme.rose; font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption
    }
}
