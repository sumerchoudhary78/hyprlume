import QtQuick
import QtQuick.Layouts
import qs.components
import qs.services
import qs.theme

// Vertical dock bound to workspaces. The reference's app icons are replaced
// by workspace tiles, so every button has a real target.
GlassSurface {
    id: root

    required property Workspaces workspaces

    radius: Theme.radiusPanel

    readonly property int maxRows: Math.max(1, Math.min(8,
        Math.floor((height - Theme.px(28)) / Theme.px(52))))

    readonly property var rows: {
        const list = (root.workspaces.all ?? []).slice();
        list.sort((a, b) => a.id - b.id);
        return list.slice(0, root.maxRows);
    }

    Column {
        anchors.top: parent.top
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.topMargin: Theme.px(14)
        spacing: Theme.px(8)

        Repeater {
            model: root.rows

            delegate: IconTile {
                required property var modelData

                id: tile

                iconName: Theme.workspaceIcon(tile.modelData && Number.isInteger(tile.modelData.id) ? tile.modelData.id - 1 : -1)
                active: tile.modelData && tile.modelData.id === root.workspaces.focusedId
                accent: Theme.amber
                size: Theme.px(44)
                iconSize: Theme.px(19)
                onClicked: {
                    if (tile.modelData)
                        root.workspaces.focus(tile.modelData);
                }
            }
        }
    }
}
