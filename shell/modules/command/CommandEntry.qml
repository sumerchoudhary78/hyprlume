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
        implicitHeight: Theme.px(44)
        radius: Theme.px(12)
        color: Theme.glassInset
        border.width: 1
        border.color: input.activeFocus ? Theme.rimStrong : Theme.rim

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
            selectedTextColor: "#081018"
            clip: true
            focus: true
            enabled: !root.blocked
            maximumLength: 512
            onAccepted: root.submit(false)
            Keys.onEscapePressed: root.dismissed()

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: input.text.length === 0
                text: "workspace 2 · focus firefox · volume 30"
                color: Theme.inkFaint
                font: input.font
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

        Chip {
            text: "ENTER"
            textColor: Theme.inkFaint
            mono: true
        }
    }
}
