import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    signal chosen()
    required property var workspaces
    required property var currentWorkspace
    readonly property var rows: workspaces.all ?? []
    readonly property int rowHeight: Theme.px(38)
    spacing: Theme.px(8)

    function revealFocused() {
        const index = rows.findIndex(workspace => workspaces.sameWorkspace(workspace, currentWorkspace));
        if (index >= 0)
            list.positionViewAtIndex(index, ListView.Contain);
    }

    onCurrentWorkspaceChanged: Qt.callLater(revealFocused)

    PanelHeader {
        Layout.fillWidth: true
        title: "Workspaces"
        IconTile {
            objectName: "nextWorkspace"
            iconName: "chevron-right"
            tooltip: "Next workspace"
            enabled: !!root.currentWorkspace
            size: Theme.px(32)
            quiet: true
            onClicked: { root.workspaces.focusNext(); root.chosen(); }
        }
    }

    ListView {
        id: list
        Layout.fillWidth: true
        Layout.preferredHeight: Math.min(count, 4) * root.rowHeight + Math.max(0, Math.min(count, 4) - 1) * spacing
        visible: count > 0
        clip: true
        spacing: Theme.px(2)
        boundsBehavior: Flickable.StopAtBounds
        model: root.rows
        onCountChanged: Qt.callLater(root.revealFocused)
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        delegate: ActionRow {
            objectName: "workspaceRow" + index
            required property var modelData
            required property int index
            readonly property int windows: root.workspaces.countFor(modelData)
            width: ListView.view.width - (ListView.view.count > 4 ? Theme.px(10) : 0)
            height: root.rowHeight
            label: root.workspaces.label(modelData)
            hint: windows === 1 ? "1 window" : windows + " windows"
            iconName: active ? "workspace-active" : "workspace"
            active: root.workspaces.sameWorkspace(modelData, root.currentWorkspace)
            quiet: true
            Accessible.name: label + ", " + hint + (active ? ", current workspace" : "")
            onActiveFocusChanged: if (activeFocus && ListView.view)
                ListView.view.positionViewAtIndex(index, ListView.Contain)
            onClicked: { root.workspaces.focus(modelData); root.chosen(); }
        }
    }

    Text {
        visible: root.rows.length === 0
        text: "No workspaces available"
        color: Theme.inkDim
        font.family: Theme.fontSans
        font.pixelSize: Theme.fontBody
    }
}
