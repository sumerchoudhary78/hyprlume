import QtQuick
import QtQuick.Layouts
import qs.components
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

    readonly property bool busy: ["planning", "executing", "speaking"].indexOf(snapshot.phase) >= 0
    readonly property bool review: snapshot.confirmation !== null && snapshot.confirmation !== undefined
    readonly property string voice: snapshot.voice ?? "off"
    readonly property string transcript: snapshot.transcript ?? ""
    readonly property string activeApp: snapshot.active_app ?? ""
    readonly property var actions: snapshot.actions ?? []
    readonly property var receipts: snapshot.receipts ?? []

    implicitWidth: Theme.px(640)
    implicitHeight: layout.implicitHeight + Theme.px(36)

    focus: true

    function submit(execute) {
        if (input.text.trim().length > 0 && !busy && !review)
            root.request({ op: "command", text: input.text.trim(), dry_run: !execute });
    }

    function focusInput() {
        input.forceActiveFocus();
    }

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
                    visible: root.activeApp.length > 0
                    text: root.activeApp
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
                    enabled: !root.busy && !root.review
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
                    enabled: input.text.trim().length > 0 && !root.busy && !root.review
                    onClicked: root.submit(false)
                }

                GlassButton {
                    text: "Run direct"
                    objectName: "runDirect"
                    enabled: input.text.trim().length > 0 && !root.busy && !root.review
                    onClicked: root.submit(true)
                }

                Item { Layout.fillWidth: true }

                Chip {
                    text: "ENTER"
                    textColor: Theme.inkFaint
                    mono: true
                }
            }

            ColumnLayout {
                visible: root.transcript.length > 0
                Layout.fillWidth: true
                spacing: Theme.px(5)

                Text {
                    text: root.voice === "dictate" ? "DICTATION · SELECT TO COPY" : "YOUR REQUEST"
                    color: Theme.inkFaint
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontMicro
                    font.letterSpacing: 1.1
                }

                TextEdit {
                    Layout.fillWidth: true
                    text: root.transcript
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

            ColumnLayout {
                visible: root.actions.length > 0
                Layout.fillWidth: true
                spacing: Theme.px(6)

                Repeater {
                    model: root.actions

                    delegate: Rectangle {
                        required property string modelData
                        required property int index

                        Layout.fillWidth: true
                        implicitHeight: Theme.px(34)
                        radius: Theme.radiusTile
                        color: Theme.glassInset
                        border.width: 1
                        border.color: Theme.divider

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Theme.px(10)
                            anchors.rightMargin: Theme.px(10)
                            spacing: Theme.px(10)

                            Text {
                                text: (index + 1).toString().padStart(2, "0")
                                color: Theme.cyan
                                font.family: Theme.fontMono
                                font.pixelSize: Theme.fontCaption
                            }

                            Text {
                                Layout.fillWidth: true
                                text: modelData
                                color: Theme.ink
                                font.family: Theme.fontSans
                                font.pixelSize: Theme.fontBody
                                elide: Text.ElideRight
                            }
                        }
                    }
                }

                RowLayout {
                    visible: root.review
                    spacing: Theme.px(8)

                    GlassButton {
                        objectName: "confirmPlan"
                    text: "Confirm"
                        primary: true
                        enabled: !root.busy
                        onClicked: root.request({ op: "confirm", token: root.snapshot.confirmation })
                    }

                    GlassButton {
                        objectName: "cancelPlan"
                        text: "Dismiss"
                        enabled: !root.busy
                        onClicked: root.request({ op: "cancel" })
                    }

                    Item { Layout.fillWidth: true }
                }
            }

            ColumnLayout {
                visible: root.receipts.length > 0
                Layout.fillWidth: true
                spacing: Theme.px(4)

                Repeater {
                    model: root.receipts

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
                    primary: root.voice === "wake"
                    onClicked: root.request({ op: "voice", mode: root.voice === "wake" ? "off" : "wake" })
                }

                GlassButton {
                    text: "Dictate"
                    compact: true
                    primary: root.voice === "dictate"
                    onClicked: root.request({ op: "voice", mode: "dictate" })
                }

                GlassButton {
                    text: "Mic off"
                    compact: true
                    primary: root.voice === "off"
                    onClicked: root.request({ op: "voice", mode: "off" })
                }

                Item { Layout.fillWidth: true }
            }

            Text {
                Layout.fillWidth: true
                text: "LOCAL VOICE  /  ENGLISH  /  " + (root.voice === "off" ? "MICROPHONE OFF" : "MICROPHONE ACTIVE")
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
