import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var workspaces
    required property var currentWorkspace
    required property var shellState
    signal chosen()
    property bool listOpen: false
    readonly property var rows: workspaces.all
    readonly property string windowIds: workspaces.atlasWindows.filter(win => !!win.id)
        .sort((a, b) => a.workspace.localeCompare(b.workspace) || a.title.localeCompare(b.title) || a.id.localeCompare(b.id))
        .map(win => win.id).join(" ")
    onWindowIdsChanged: if (!windowIds) listOpen = false
    spacing: Theme.px(12)
    function focusCard(index) {
        const card = cards.itemAt(Math.max(0, Math.min(cards.count - 1, index)));
        if (card) card.forceActiveFocus(Qt.TabFocusReason);
    }
    Component.onCompleted: Qt.callLater(() => {
        const index = rows.findIndex(workspace => workspaces.sameWorkspace(workspace, currentWorkspace));
        if (index >= 0 && !listOpen) focusCard(index);
    })

    RowLayout {
        Layout.fillWidth: true
        Text {
            Layout.fillWidth: true; Layout.minimumWidth: 0
            text: root.rows.length + (root.rows.length === 1 ? " workspace" : " workspaces")
            color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: Theme.fontBody
        }
        GlassButton {
            objectName: "atlasList"; text: root.listOpen ? "Show maps" : "Window list"
            compact: true; enabled: !!root.windowIds
            Accessible.checkable: true; Accessible.checked: root.listOpen
            onClicked: root.listOpen = !root.listOpen
        }
        IconTile {
            objectName: "nextWorkspace"; iconName: "chevron-right"; tooltip: "Next workspace"
            enabled: !!root.currentWorkspace; size: Theme.px(32); quiet: true
            onClicked: { root.workspaces.focusNext(); root.chosen(); }
        }
    }

    Flow {
        id: grid
        Layout.fillWidth: true; visible: !root.listOpen && root.rows.length > 0
        readonly property int columns: Math.max(1, Math.min(4, Math.floor((width + spacing) / Theme.px(146))))
        spacing: Theme.px(12)
        Repeater {
            id: cards
            model: grid.visible ? root.rows : []
            AtlasCard {
                required property int index
                required property var modelData
                objectName: "workspaceRow" + index
                width: Math.floor((grid.width - grid.spacing * (grid.columns - 1)) / grid.columns)
                plotHeight: Theme.px(96)
                workspace: modelData; workspaces: root.workspaces; shellState: root.shellState
                current: root.workspaces.sameWorkspace(modelData, root.currentWorkspace)
                onClicked: { root.workspaces.focus(modelData); root.chosen(); }
                Keys.onLeftPressed: root.focusCard(index - 1)
                Keys.onRightPressed: root.focusCard(index + 1)
                Keys.onUpPressed: root.focusCard(index - grid.columns)
                Keys.onDownPressed: root.focusCard(index + grid.columns)
                Keys.onPressed: event => {
                    if (event.key !== Qt.Key_Home && event.key !== Qt.Key_End) return;
                    root.focusCard(event.key === Qt.Key_Home ? 0 : cards.count - 1);
                    event.accepted = true;
                }
            }
        }
    }
    Repeater {
        // Window resizes leave the keyboard list and its focused button in place.
        model: root.listOpen && root.windowIds ? root.windowIds.split(" ") : []
        ActionRow {
            required property string modelData
            readonly property var entry: root.workspaces.atlasWindows.find(win => win.id === modelData)
            objectName: "atlasListWindow" + modelData
            Layout.fillWidth: true
            label: entry?.title ?? ""; hint: entry?.workspace ?? ""; iconName: "workspace"
            enabled: !!entry && root.shellState.backend.online && !root.shellState.windowRequest
            Accessible.name: label + ", " + hint
            onClicked: root.shellState.focusWindow(entry)
        }
    }
    Text {
        Layout.fillWidth: true
        visible: !root.rows.length || !!root.shellState.windowError
        text: root.shellState.windowError || "Workspace information is unavailable"
        textFormat: Text.PlainText; wrapMode: Text.Wrap
        color: root.shellState.windowError ? Theme.rose : Theme.inkDim
        font.family: Theme.fontSans; font.pixelSize: Theme.fontBody
    }
}
