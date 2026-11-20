import QtQuick
import QtQuick.Layouts
import qs.components
import qs.models
import qs.theme

GlassSurface {
    id: root
    required property var backend
    property bool condensed: false

    readonly property int padding: Theme.px(condensed ? 16 : 26)
    readonly property var snapshot: backend.snapshot
    readonly property bool available: backend.online && !commandState.blocked && !backend.locking.active
    readonly property color statusColor: !backend.online || snapshot.phase === "error" ? Theme.rose
        : commandState.busy || commandState.review ? Theme.amber : Theme.mint
    readonly property string status: !backend.online ? "Local agent offline"
        : commandState.review ? "Review ready"
        : commandState.busy ? "Command in progress"
        : snapshot.phase === "error" ? "Command failed"
        : "Local agent ready"
    readonly property string activity: commandState.receipts[0] ?? (snapshot.phase === "ready" ? "" : snapshot.message ?? "")

    implicitHeight: body.implicitHeight + padding * 2
    radius: Theme.radiusPanel

    CommandState { id: commandState; snapshot: root.snapshot }

    function preview(text) {
        if (!available || !text.trim())
            return;
        backend.send({op: "command", text: text.trim(), dry_run: true});
    }

    ColumnLayout {
        id: body
        anchors.fill: parent
        anchors.margins: root.padding
        spacing: Theme.px(root.condensed ? 12 : 18)

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(12)

            Rectangle {
                implicitWidth: Theme.px(39); implicitHeight: implicitWidth
                radius: Theme.px(11)
                color: Theme.selection
                Icon { anchors.centerIn: parent; name: "utilities-terminal"; color: Theme.cyan }
            }

            Text {
                Layout.fillWidth: true
                text: "Desktop command"
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontHeadline
                font.weight: Font.DemiBold
                color: Theme.ink
                elide: Text.ElideRight
                Accessible.role: Accessible.Heading
            }
        }

        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Theme.px(58)
            radius: Theme.px(12)
            color: Theme.field
            border.width: input.activeFocus ? 2 : 1
            border.color: input.activeFocus ? Theme.cyan : Theme.rim

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.px(16)
                anchors.rightMargin: Theme.px(7)
                spacing: Theme.px(8)

                TextInput {
                    id: input
                    objectName: "consoleInput"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    verticalAlignment: TextInput.AlignVCenter
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontLabel
                    color: Theme.ink
                    selectionColor: Theme.cyan
                    selectedTextColor: Theme.onAccent
                    enabled: root.available
                    activeFocusOnTab: true
                    selectByMouse: true
                    maximumLength: 512
                    clip: true
                    Accessible.name: "Desktop command"
                    Accessible.description: "Press Enter to preview the action"

                    onAccepted: {
                        if (text.trim().length > 0) {
                            root.preview(text);
                            clear();
                        }
                    }

                    Text {
                        anchors.fill: parent
                        verticalAlignment: Text.AlignVCenter
                        visible: !input.text
                        text: "Type a command…"
                        color: Theme.inkFaint
                        font: input.font
                        elide: Text.ElideRight
                    }
                }

                GlassButton {
                    objectName: "consolePreview"
                    text: "Preview"
                    primary: true
                    enabled: root.available && input.text.trim().length > 0
                    onClicked: input.accepted()
                }
            }
        }

        Flow {
            Layout.fillWidth: true
            spacing: Theme.px(7)

            GlassButton {
                objectName: "launcherterminal"
                text: "Open terminal"
                iconName: "utilities-terminal"
                compact: true; quiet: true
                enabled: root.available
                onClicked: root.preview("launch terminal")
            }

            GlassButton {
                objectName: "agentVolume"
                text: "Volume 35%"
                iconName: "audio-volume-high"
                compact: true; quiet: true
                enabled: root.available
                onClicked: root.preview("volume 35")
            }

            GlassButton {
                objectName: "dictateNote"
                text: "Dictate"
                iconName: "audio-input-microphone"
                compact: true; quiet: true
                enabled: root.backend.online && !root.backend.locking.active
                onClicked: root.backend.send({op: "voice", mode: "dictate"})
            }
        }

        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(7)

            StatusDot { color: root.statusColor; size: Theme.px(6); halo: false }

            Text {
                Layout.fillWidth: true
                text: root.status
                color: Theme.inkDim
                font.family: Theme.fontSans
                font.pixelSize: Theme.fontCaption
                elide: Text.ElideRight
            }

            GlassButton {
                visible: commandState.review && !root.snapshot.panel_open
                text: "Review plan"
                compact: true
                onClicked: root.backend.send({op: "toggle"})
            }
        }

        Text {
            visible: !!root.activity
            Layout.fillWidth: true
            text: root.activity
            textFormat: Text.PlainText
            color: Theme.inkDim
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontBody
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
        }
    }
}
