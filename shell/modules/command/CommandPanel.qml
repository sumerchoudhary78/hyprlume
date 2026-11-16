import QtQuick
import QtQuick.Layouts
import qs.components
import qs.models
import qs.theme

// Command palette. This is the live surface for the daemon's plan/confirm
// flow; it uses the same glass primitives and tokens as the dashboard.
Item {
    id: root

    property var snapshot: ({
        phase: "ready",
        message: "Your desktop, within reach.",
        transcript: "",
        actions: [],
        receipts: [],
        confirmation: null,
        panel_open: true,
        voice: "off",
        workspace: null,
        active_app: ""
    })
    property string connectionError: ""

    signal request(var operation)
    signal dismissed()

    CommandState {
        id: commandState
        snapshot: root.snapshot
    }

    implicitWidth: Theme.px(640)
    implicitHeight: layout.implicitHeight + Theme.px(36)

    focus: true

    function focusInput() { entry.focusInput(); }

    // Swallow clicks on the card so the scrim behind it does not dismiss the
    // palette when the user clicks empty glass.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
    }

    GlassSurface {
        anchors.fill: parent
        radius: Theme.radiusPanel

        ColumnLayout {
            id: layout

            anchors.fill: parent
            anchors.margins: Theme.px(18)
            spacing: Theme.px(12)

            PanelHeader {
                Layout.fillWidth: true
                title: "Command"
                iconName: "utilities-terminal"
                accent: Theme.amber

                GlassButton {
                    text: "×"
                    compact: true
                    Accessible.name: "Close command panel"
                    onClicked: root.dismissed()
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.px(8)

                StatusDot {
                    color: root.snapshot.phase === "error" ? Theme.rose : Theme.mint
                    size: Theme.px(7)
                }

                Text {
                    text: (root.snapshot.phase || "ready").toUpperCase()
                    color: Theme.inkDim
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.fontCaption
                    font.letterSpacing: 1.2
                }

                Item { Layout.fillWidth: true }

                Chip {
                    visible: commandState.activeApp.length > 0
                    text: commandState.activeApp
                    textColor: Theme.inkFaint
                    mono: true
                }

                Chip {
                    text: root.snapshot.workspace !== null && root.snapshot.workspace !== undefined
                        ? "WS " + root.snapshot.workspace
                        : "NO WS"
                    textColor: Theme.inkFaint
                    mono: true
                }
            }

            Text {
                Layout.fillWidth: true
                text: root.snapshot.message
                color: Theme.ink
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontTitle
                font.weight: Font.DemiBold
                wrapMode: Text.Wrap
            }

            CommandEntry {
                id: entry
                Layout.fillWidth: true
                blocked: commandState.blocked
                onSubmitted: (text, execute) => root.request({op: "command", text: text, dry_run: !execute})
                onDismissed: root.dismissed()
            }

            ColumnLayout {
                visible: commandState.transcript.length > 0
                Layout.fillWidth: true
                spacing: Theme.px(5)

                Text {
                    text: commandState.voice === "dictate" ? "DICTATION · SELECT TO COPY" : "YOUR REQUEST"
                    color: Theme.inkFaint
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontMicro
                    font.letterSpacing: 1.1
                }

                TextEdit {
                    Layout.fillWidth: true
                    text: commandState.transcript
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextEdit.Wrap
                    color: Theme.inkDim
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontBody
                    padding: 0
                    Layout.preferredHeight: contentHeight
                }
            }

            PlanReview {
                Layout.fillWidth: true
                actions: commandState.actions
                busy: commandState.busy
                review: commandState.review
                onConfirmed: root.request({op: "confirm", token: root.snapshot.confirmation})
                onCancelled: root.request({op: "cancel"})
            }

            ColumnLayout {
                visible: commandState.receipts.length > 0
                Layout.fillWidth: true
                spacing: Theme.px(4)

                Repeater {
                    model: commandState.receipts

                    delegate: Text {
                        required property string modelData

                        Layout.fillWidth: true
                        text: "✓  " + modelData
                        color: Theme.mint
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.fontBody
                        wrapMode: Text.Wrap
                    }
                }
            }

            Text {
                visible: root.connectionError.length > 0
                Layout.fillWidth: true
                text: root.connectionError
                color: Theme.rose
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontBody
                wrapMode: Text.Wrap
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 1
                color: Theme.divider
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.px(6)

                GlassButton {
                    text: "Wake word"
                    compact: true
                    primary: commandState.voice === "wake"
                    onClicked: root.request({ op: "voice", mode: commandState.voice === "wake" ? "off" : "wake" })
                }

                GlassButton {
                    text: "Dictate"
                    compact: true
                    primary: commandState.voice === "dictate"
                    onClicked: root.request({ op: "voice", mode: "dictate" })
                }

                GlassButton {
                    text: "Mic off"
                    compact: true
                    primary: commandState.voice === "off"
                    onClicked: root.request({ op: "voice", mode: "off" })
                }

                Item { Layout.fillWidth: true }
            }

            Text {
                Layout.fillWidth: true
                text: "LOCAL VOICE  /  ENGLISH  /  " + (commandState.voice === "off" ? "MICROPHONE OFF" : "MICROPHONE ACTIVE")
                color: Theme.inkFaint
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontMicro
                font.letterSpacing: 1
                wrapMode: Text.Wrap
            }
        }
    }

    Keys.onEscapePressed: root.dismissed()

    Timer {
        interval: 60
        running: true
        repeat: false
        onTriggered: root.focusInput()
    }

    Component.onCompleted: root.focusInput()
}
