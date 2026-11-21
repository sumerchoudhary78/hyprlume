import QtQuick
import QtQuick.Controls
import QtQuick.Templates as T
import qs.theme

T.Button {
    id: card
    required property var workspaces
    required property var workspace
    required property var shellState
    required property bool current
    property bool ambient: false
    property int plotHeight: Theme.px(92)
    readonly property var windows: workspaces.windowsFor(workspace)
    readonly property string caption: workspaces.label(workspace)
    readonly property string countLabel: windows.length === 1 ? "1 window" : windows.length + " windows"
    implicitHeight: plotHeight + Theme.px(48)
    padding: 0
    hoverEnabled: true
    focusPolicy: ambient ? Qt.NoFocus : Qt.TabFocus
    Keys.onReturnPressed: click()
    Accessible.name: caption + ", " + countLabel + (current ? ", current workspace" : "")
    Accessible.description: "Switch to this workspace"
    background: null

    contentItem: Item {
        Rectangle {
            id: plot
            anchors { top: parent.top; left: parent.left; right: parent.right }
            height: card.plotHeight
            radius: Theme.px(4)
            color: Theme.highContrast ? "#0a1012" : card.current ? "#132923" : "#142528"
            border.width: card.visualFocus || card.current ? 2 : 1
            border.color: card.visualFocus ? Theme.focusRing : card.current ? Theme.atlasCopper : card.hovered ? Theme.atlasPaper : "#66807c"
            Canvas {
                anchors.fill: parent; anchors.margins: 3
                onPaint: {
                    const ctx = getContext("2d");
                    ctx.clearRect(0, 0, width, height);
                    ctx.fillStyle = "#45615b";
                    const pitch = Theme.px(12);
                    for (let y = pitch / 2; y < height; y += pitch)
                        for (let x = pitch / 2; x < width; x += pitch) ctx.fillRect(x, y, 1, 1);
                }
            }
            Item {
                id: viewport
                readonly property real aspect: card.workspaces.aspectFor(card.workspace)
                anchors.centerIn: parent
                width: Math.min(plot.width - Theme.px(10), (plot.height - Theme.px(10)) * aspect)
                height: width / aspect
                clip: true
                Repeater {
                    model: card.windows
                    delegate: Rectangle {
                        id: pane
                        required property var modelData
                        readonly property var frame: modelData.frame
                        readonly property string appName: modelData.app.split(".").pop() || modelData.app
                        readonly property color tint: {
                            const colors = ["#a7c8af", "#94c4c4", "#e1b386", "#a6b2d0", "#c7b78a"];
                            let hash = 0;
                            for (const character of modelData.app) hash = (hash * 31 + character.charCodeAt(0)) | 0;
                            return colors[Math.abs(hash) % colors.length];
                        }
                        x: frame ? frame.x * viewport.width : 0
                        y: frame ? frame.y * viewport.height : 0
                        width: frame ? frame.w * viewport.width : 0
                        height: frame ? frame.h * viewport.height : 0
                        visible: !!frame
                        radius: Theme.px(2); clip: true
                        color: Theme.highContrast ? "#152023" : Qt.tint("#162726", Qt.rgba(tint.r, tint.g, tint.b, hit.containsMouse ? 0.49 : 0.25))
                        border.color: hit.containsMouse || modelData.active ? Theme.atlasPaper : tint
                        border.width: hit.containsMouse || modelData.active ? 2 : 1
                        Rectangle {
                            anchors { top: parent.top; left: parent.left; right: parent.right; margins: 2 }
                            height: Theme.px(3); color: pane.tint; opacity: 0.75
                        }
                        TextMetrics {
                            id: appMetrics
                            font.family: Theme.fontSans; font.pixelSize: Theme.px(10)
                            text: pane.appName
                        }
                        Text {
                            anchors.centerIn: parent; width: parent.width - Theme.px(6)
                            visible: parent.width > Theme.px(31) && parent.height > Theme.px(20)
                            text: appMetrics.advanceWidth <= width ? pane.appName : pane.appName.charAt(0).toUpperCase()
                            textFormat: Text.PlainText
                            horizontalAlignment: Text.AlignHCenter
                            font.family: Theme.fontSans; font.pixelSize: Theme.px(10); color: Theme.atlasPaper
                        }
                        MouseArea {
                            id: hit
                            objectName: (card.ambient ? "homeWindow" : "atlasWindow") + pane.modelData.id
                            anchors.fill: parent
                            enabled: !!pane.modelData.id && card.shellState.backend.online && width >= Theme.px(24) && height >= Theme.px(24)
                            hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: card.shellState.focusWindow(pane.modelData)
                            ToolTip {
                                visible: hit.containsMouse
                                delay: 450
                                text: pane.modelData.title
                                width: Math.min(implicitWidth, Theme.px(280))
                                horizontalPadding: Theme.px(10); verticalPadding: Theme.px(6)
                                background: Rectangle {
                                    color: "#132527"; radius: Theme.px(5)
                                    border.color: Theme.atlasCopper
                                }
                                contentItem: Text {
                                    text: pane.modelData.title; textFormat: Text.PlainText
                                    color: Theme.atlasPaper; font.family: Theme.fontSans
                                    font.pixelSize: Theme.fontCaption; elide: Text.ElideRight
                                }
                            }
                        }
                    }
                }
            }
            Text {
                anchors.centerIn: parent
                visible: !card.windows.some(win => !!win.frame)
                text: card.windows.length ? "No layout" : "Empty"
                font.family: Theme.fontMono; font.pixelSize: Theme.px(11)
                color: Theme.inkDim
            }
        }
        Text {
            id: nameLabel
            anchors { left: parent.left; right: parent.right; top: plot.bottom; topMargin: Theme.px(7) }
            text: card.caption; textFormat: Text.PlainText; elide: Text.ElideRight
            color: Theme.atlasPaper; font.family: Theme.fontSans; font.pixelSize: Theme.px(14); font.weight: Font.DemiBold
        }
        Text {
            anchors { left: parent.left; right: parent.right; top: nameLabel.bottom; topMargin: Theme.px(1) }
            text: card.current ? "Current · " + card.countLabel : card.countLabel
            textFormat: Text.PlainText; elide: Text.ElideRight
            color: card.current ? Theme.atlasCopper : Theme.inkDim
            font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption
        }
    }
}
