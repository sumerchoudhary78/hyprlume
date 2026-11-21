import QtQuick
import qs.components
import qs.theme

Item {
    id: root
    required property var workspaces
    required property var currentWorkspace
    required property var shellState
    property var outputScreen: null
    readonly property rect mapTargets: Qt.rect(content.x, content.y + actions.y + maps.y, maps.width, maps.height)
    readonly property rect buttonTargets: Qt.rect(content.x, content.y + actions.y + buttons.y, buttons.width, buttons.height)
    readonly property int inset: Math.round(Math.min(Theme.px(98), Math.max(Theme.px(20), width * 0.055)))
    readonly property bool compact: width < Theme.px(680)
    readonly property bool firstWorkspace: workspaces.all.length === 1 && !workspaces.atlasWindows.length && shellState.backend.online
    readonly property int slots: Math.max(1, Math.min(4, Math.floor((content.width + Theme.px(12)) / Theme.px(138))))
    readonly property var visibleWorkspaces: {
        const rows = workspaces.all;
        const index = rows.findIndex(workspace => workspaces.sameWorkspace(workspace, currentWorkspace));
        const start = Math.max(0, Math.min(index, rows.length - slots));
        return rows.slice(start, start + slots);
    }

    Rectangle {
        id: shade
        anchors { left: parent.left; top: parent.top; bottom: parent.bottom }
        width: Math.min(root.width, content.width + root.inset * 4)
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: Theme.highContrast ? "#fd081115" : "#e10b1719" }
            GradientStop {
                position: Math.min(0.98, (content.x + content.width) / Math.max(1, shade.width))
                color: Theme.highContrast ? "#f8081115" : "#cf0b1719"
            }
            GradientStop { position: 1; color: "transparent" }
        }
    }
    Column {
        id: content
        x: root.inset
        y: Math.max(Theme.barHeight + Theme.px(12), Math.min(root.height - height - Theme.px(14), Math.round((root.height - height) * 0.48)))
        width: Math.min(root.width - root.inset * 2, Math.max(root.width * 0.43, Theme.px(552)), Theme.px(720))
        spacing: Theme.px(root.compact ? 10 : 15)
        Row {
            spacing: Theme.px(10)
            Icon { anchors.verticalCenter: parent.verticalCenter; name: "compass"; size: Theme.px(20); color: Theme.atlasCopper }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.workspaces.all.length + (root.workspaces.all.length === 1 ? " WORKSPACE" : " WORKSPACES")
                    + "  /  " + root.workspaces.atlasWindows.length + (root.workspaces.atlasWindows.length === 1 ? " WINDOW" : " WINDOWS")
                font.family: Theme.fontMono; font.pixelSize: Theme.px(11); font.letterSpacing: Theme.px(1)
                color: Theme.atlasPaper
            }
        }
        Text {
            text: "Workspace atlas"
            font.family: Theme.fontAtlas; font.pixelSize: Theme.px(root.compact ? 57 : 82)
            color: Theme.atlasPaper
            Accessible.role: Accessible.Heading
        }
        Column {
            id: actions
            width: parent.width; spacing: Theme.px(16)
            Rectangle { width: parent.width; height: 1; color: "#677e77" }
            Row {
                id: maps
                spacing: Theme.px(12)
                Repeater {
                    model: root.visibleWorkspaces
                    AtlasCard {
                        required property int index
                        required property var modelData
                        objectName: "homeWorkspace" + index
                        width: Math.floor((content.width - (root.slots - 1) * Theme.px(12)) / root.slots)
                        plotHeight: Theme.px(root.compact ? 68 : 80)
                        workspace: modelData; workspaces: root.workspaces; shellState: root.shellState
                        current: root.workspaces.sameWorkspace(modelData, root.currentWorkspace)
                        ambient: true
                        onClicked: root.workspaces.focus(modelData)
                    }
                }
            }
            Row {
                id: buttons
                spacing: Theme.px(8)
                GlassButton {
                    objectName: "expandAtlas"; text: "Open atlas"; iconName: "expand"
                    primary: !root.firstWorkspace; accent: Theme.atlasCopper; focusPolicy: Qt.NoFocus
                    onClicked: root.shellState.toggleMenu("overview", root.outputScreen)
                }
                GlassButton {
                    objectName: "atlasApps"; text: root.firstWorkspace ? "Open apps" : "Apps"; iconName: "grid"; focusPolicy: Qt.NoFocus
                    primary: root.firstWorkspace; accent: Theme.atlasCopper
                    enabled: root.shellState.backend.online
                    onClicked: root.shellState.openFromBar(root.outputScreen, {op:"panel", panel:"launcher", open:true})
                }
            }
            Text {
                width: parent.width; visible: !!text
                text: root.shellState.windowError; textFormat: Text.PlainText; wrapMode: Text.Wrap
                color: Theme.rose; font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption
            }
        }
    }
}
