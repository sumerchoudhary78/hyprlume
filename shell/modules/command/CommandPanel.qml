import QtQuick
import QtQuick.Layouts
import qs.components
import qs.models
import qs.theme

FocusScope {
    id: root

    required property var snapshot
    property string connectionError: ""

    signal request(var operation)
    signal dismissed()

    CommandState { id: commandState; snapshot: root.snapshot }

    implicitWidth: Theme.px(640)
    implicitHeight: layout.implicitHeight + Theme.px(40)
    focus: true
    Keys.onEscapePressed: root.dismissed()

    // Keep clicks on the card away from the background dismiss area.
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }

    GlassSurface {
        anchors.fill: parent
        radius: Theme.radiusPanel

        ColumnLayout {
            id: layout
            anchors.fill: parent
            anchors.margins: Theme.px(20)
            spacing: Theme.px(14)

            PanelHeader {
                Layout.fillWidth: true
                title: commandState.review ? "Review command" : "Command"
                iconName: "utilities-terminal"

                Text {
                    visible: root.snapshot.workspace !== null && root.snapshot.workspace !== undefined
                    Layout.maximumWidth: Theme.px(210)
                    text: "Workspace " + root.snapshot.workspace + (commandState.activeApp ? " · " + commandState.activeApp : "")
                    textFormat: Text.PlainText
                    color: Theme.inkFaint
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontCaption
                    elide: Text.ElideRight
                }

                IconTile {
                    iconName: "close"
                    tooltip: "Close command panel"
                    quiet: true
                    size: Theme.px(36)
                    onClicked: root.dismissed()
                }
            }

            FocusScroll {
                id: scroll
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredHeight: body.implicitHeight
                contentWidth: width
                contentHeight: body.implicitHeight

                ColumnLayout {
                    id: body
                    width: scroll.width - (scroll.interactive ? Theme.px(12) : 0)
                    spacing: Theme.px(14)

                    Text {
                        Layout.fillWidth: true
                        text: root.snapshot.message ?? ""
                        visible: !!text && (!commandState.review || root.snapshot.phase === "error")
                        textFormat: Text.PlainText
                        color: root.snapshot.phase === "error" ? Theme.rose : Theme.inkDim
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.fontBody
                        wrapMode: Text.Wrap
                    }

                    CommandEntry {
                        id: entry
                        Layout.fillWidth: true
                        visible: !commandState.review
                        blocked: commandState.blocked
                        onSubmitted: (text, execute) => root.request({op: "command", text: text, dry_run: !execute})
                        onDismissed: root.dismissed()
                    }

                    ColumnLayout {
                        visible: commandState.transcript.length > 0
                        Layout.fillWidth: true
                        spacing: Theme.px(4)

                        Text {
                            text: commandState.voice === "dictate" ? "Dictation" : "Request"
                            color: Theme.inkFaint
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.fontCaption
                        }

                        TextEdit {
                            Layout.fillWidth: true
                            Layout.preferredHeight: contentHeight
                            text: commandState.transcript
                            textFormat: TextEdit.PlainText
                            readOnly: true
                            selectByMouse: true
                            activeFocusOnTab: true
                            Accessible.name: commandState.voice === "dictate" ? "Transcribed text" : "Command request"
                            wrapMode: TextEdit.Wrap
                            color: Theme.ink
                            selectionColor: Theme.cyan
                            selectedTextColor: Theme.onAccent
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.fontBody
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

                    Repeater {
                        model: commandState.receipts
                        delegate: Text {
                            required property string modelData
                            Layout.fillWidth: true
                            text: "✓  " + modelData
                            textFormat: Text.PlainText
                            color: Theme.mint
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.fontBody
                            wrapMode: Text.Wrap
                        }
                    }

                    Text {
                        visible: !!root.connectionError
                        Layout.fillWidth: true
                        text: root.connectionError
                        textFormat: Text.PlainText
                        color: Theme.rose
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.fontBody
                        wrapMode: Text.Wrap
                    }
                }

            }

            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.px(5)

                GlassButton {
                    text: "Wake word"
                    compact: true
                    primary: commandState.voice === "wake"
                    onClicked: root.request({op: "voice", mode: commandState.voice === "wake" ? "off" : "wake"})
                }

                GlassButton {
                    text: "Dictate"
                    compact: true
                    primary: commandState.voice === "dictate"
                    onClicked: root.request({op: "voice", mode: "dictate"})
                }

                Item { Layout.fillWidth: true }

                GlassButton {
                    iconName: "microphone-off"
                    text: commandState.voice === "off" ? "Mic off" : "Turn mic off"
                    compact: true
                    enabled: commandState.voice !== "off"
                    onClicked: root.request({op: "voice", mode: "off"})
                }
            }
        }
    }

    Timer {
        interval: 60
        running: true
        onTriggered: if (!commandState.review) entry.focusInput()
    }

    Component.onCompleted: if (!commandState.review) entry.focusInput()
}
