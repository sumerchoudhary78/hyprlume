import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

// Local editor state and keyboard semantics; emits intent without IPC access.
ColumnLayout {
    id: root
    property bool blocked: false
    signal submitted(string text, bool execute)
    signal dismissed()
    spacing: Theme.px(12)

    function submit(execute) {
        const value = input.text.trim();
        if (value.length > 0 && !blocked) submitted(value, execute);
    }

    function focusInput() { input.forceActiveFocus(); }

    Rectangle {
        Layout.fillWidth: true
        implicitHeight: Theme.px(48)
        radius: Theme.px(12)
        color: Theme.field
        border.width: input.activeFocus ? 2 : 1
        border.color: input.activeFocus ? Theme.focusRing : Theme.rim

        TextInput {
            id: input
            objectName: "commandInput"

            anchors.fill: parent
            anchors.leftMargin: Theme.px(14)
            anchors.rightMargin: Theme.px(14)
            verticalAlignment: TextInput.AlignVCenter
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontBody
            selectionColor: Theme.cyan
            selectedTextColor: Theme.onAccent
            clip: true
            selectByMouse: true
            activeFocusOnTab: true
            focus: true
            enabled: !root.blocked
            maximumLength: 512
            Accessible.name: "Desktop command"
            Accessible.description: "Press Enter to preview the action"
            onAccepted: root.submit(false)
            Keys.onEscapePressed: root.dismissed()

            Text {
                anchors.fill: parent
                verticalAlignment: Text.AlignVCenter
                visible: input.text.length === 0
                text: "Try “workspace 2”"
                color: Theme.inkFaint
                font: input.font
                elide: Text.ElideRight
            }
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.px(8)

        GlassButton {
            text: "Preview"
            objectName: "previewCommand"
            primary: true
            enabled: input.text.trim().length > 0 && !root.blocked
            onClicked: root.submit(false)
        }

        GlassButton {
            text: "Run direct"
            objectName: "runDirect"
            enabled: input.text.trim().length > 0 && !root.blocked
            onClicked: root.submit(true)
        }

        Item { Layout.fillWidth: true }

        Text {
            visible: !root.blocked && root.width >= Theme.px(272)
            text: "Enter to preview"
            color: Theme.inkFaint
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontCaption
        }
    }
}
