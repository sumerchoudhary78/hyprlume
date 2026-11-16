import QtQuick
import qs.components
import qs.services
import qs.theme
import Quickshell.Widgets

// The centre stage: a console for the shell's own activity, with the
// procedural horizon underneath. Text comes from the daemon snapshot, not
// from a mock log.
Item {
    id: root

    required property Backend backend
    required property Workspaces workspaces
    required property Metrics metrics
    required property Clock clock

    readonly property var snapshot: backend.snapshot
    readonly property string phase: (snapshot.phase ?? "offline").toUpperCase()

    readonly property var output: {
        const state = root.snapshot;
        const lines = [];
        (state.actions ?? []).forEach((action, index) => {
            const number = index + 1;
            lines.push({
                kind: "action",
                text: (number < 10 ? "0" : "") + number + "   " + action
            });
        });
        (state.receipts ?? []).forEach(receipt => {
            lines.push({ kind: "ok", text: "✓   " + receipt });
        });
        return lines.slice(0, 7);
    }

    GlassSurface {
        anchors.fill: parent
        radius: Theme.radiusPanel

        // -- Backdrop --------------------------------------------------------
        ClippingRectangle {
            anchors {
                left: parent.left
                right: parent.right
                bottom: parent.bottom
            }
            height: Math.round(parent.height * 0.46)
            color: "transparent"
            topLeftRadius: 0
            topRightRadius: 0
            bottomLeftRadius: Theme.radiusPanel
            bottomRightRadius: Theme.radiusPanel

            StageBackdrop { anchors.fill: parent }

            Rectangle {
                anchors.fill: parent
                gradient: Gradient {
                    GradientStop { position: 0.0; color: Theme.glass }
                    GradientStop { position: 0.32; color: "transparent" }
                }
            }
        }

        // -- Header ----------------------------------------------------------
        Row {
            id: tabs

            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                leftMargin: Theme.px(18)
                rightMargin: Theme.px(18)
                topMargin: Theme.px(14)
            }
            spacing: Theme.px(8)

            Chip {
                text: "~/" + (root.snapshot.active_app.length > 0 ? root.snapshot.active_app : "shell")
                textColor: Theme.ink
                fillColor: Qt.rgba(0.37, 0.85, 1.0, 0.10)
                strokeColor: Qt.rgba(0.37, 0.85, 1.0, 0.22)
                horizontalPadding: Theme.px(12)
            }

            Chip {
                text: root.snapshot.workspace !== null
                    ? "workspace " + root.snapshot.workspace
                    : "no workspace"
                textColor: Theme.inkDim
            }

            Item { width: Theme.px(4); height: 1 }

            Chip {
                text: root.phase
                textColor: root.snapshot.phase === "error" ? Theme.rose
                    : root.snapshot.phase === "ready" ? Theme.mint
                    : Theme.amber
                fillColor: "transparent"
                strokeColor: Theme.divider
                mono: true
            }
        }

        // -- Prompt ----------------------------------------------------------
        Item {
            id: prompt

            anchors {
                left: parent.left
                right: parent.right
                top: tabs.bottom
                leftMargin: Theme.px(18)
                rightMargin: Theme.px(18)
                topMargin: Theme.px(12)
            }
            height: Theme.px(34)

            Rectangle {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: promptLabel.implicitWidth + Theme.px(20)
                height: Theme.px(26)
                radius: Theme.radiusPill
                color: Qt.rgba(0.64, 0.55, 1.0, 0.16)
                border.width: 1
                border.color: Qt.rgba(0.64, 0.55, 1.0, 0.35)

                Text {
                    id: promptLabel
                    anchors.centerIn: parent
                    text: "❯ " + (root.snapshot.transcript.length > 0
                        ? root.snapshot.transcript
                        : "awaiting a command")
                    color: Theme.ink
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.fontBody
                    elide: Text.ElideRight
                    width: Math.min(implicitWidth, root.width - Theme.px(260))
                }
            }

            Row {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.px(6)

                Chip {
                    text: root.workspaces.focusedWindowCount + " windows"
                    textColor: Theme.inkFaint
                }

                Chip {
                    text: root.backend.online ? "daemon online" : "daemon offline"
                    textColor: root.backend.online ? Theme.mint : Theme.rose
                    fillColor: "transparent"
                    strokeColor: Theme.divider
                }
            }
        }

        // -- Output ----------------------------------------------------------
        Column {
            id: outputArea

            anchors {
                left: parent.left
                right: parent.right
                top: prompt.bottom
                leftMargin: Theme.px(18)
                rightMargin: Theme.px(210)
                topMargin: Theme.px(10)
            }
            spacing: Theme.px(6)
            clip: true

            Repeater {
                model: root.output

                Text {
                    required property var modelData

                    width: outputArea.width
                    text: modelData.text
                    color: modelData.kind === "ok" ? Theme.mint : Theme.inkDim
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.fontBody
                    wrapMode: Text.Wrap
                    elide: Text.ElideRight
                    maximumLineCount: 2
                }
            }

            Column {
                visible: root.output.length === 0
                spacing: Theme.px(6)

                Text {
                    text: root.snapshot.message
                    width: outputArea.width
                    color: Theme.ink
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontTitle
                    font.weight: Font.Medium
                    wrapMode: Text.Wrap
                }

                Text {
                    text: "Type a direct command, or ask in plain English. Model plans always preview first."
                    width: Math.min(outputArea.width, Theme.px(520))
                    color: Theme.inkFaint
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontCaption
                    wrapMode: Text.Wrap
                }
            }
        }

        ClockCard {
            anchors {
                right: parent.right
                top: parent.top
                rightMargin: Theme.px(16)
                topMargin: Theme.px(58)
            }
            clock: root.clock
            metrics: root.metrics
            visible: root.width > Theme.px(620)
        }
    }
}
