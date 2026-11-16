import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var workspaces
    readonly property var rows: (workspaces.all ?? []).slice(0, 4)
    glowColor: Theme.violet
    secondaryColor: Theme.cyan
    PanelHeader {
        x: 17; y: 15; width: parent.width - 34
        title: "Workspaces"; iconName: "layers"; accent: Theme.cyan
        IconTile { iconName: "plus"; size: 22; iconSize: 14; onClicked: root.workspaces.focusNext() }
    }
    Column {
        x: 14; y: 47; width: parent.width - 28; spacing: 3
        Repeater {
            model: root.rows
            delegate: Item {
                id: row
                objectName: "workspaceRow" + index
                required property var modelData
                required property int index
                readonly property bool current: root.workspaces.isFocused(modelData)
                width: parent.width; height: 30
                Rectangle {
                    anchors.fill: parent; radius: 12
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0; color: row.current ? "#80543fa1" : "#18131a29" }
                        GradientStop { position: 1; color: row.current ? "#4b133d6c" : "#10091020" }
                    }
                    border.width: row.current ? 1 : 0
                    border.color: Theme.violet
                }
                RowLayout {
                    anchors.fill: parent; anchors.leftMargin: 12; anchors.rightMargin: 9; spacing: 11
                    StatusDot { size: 7; color: row.current ? Theme.cyan : row.index === 1 ? Theme.mint : row.index === 2 ? Theme.amber : Theme.mint; halo: row.current }
                    Text { Layout.fillWidth: true; text: root.workspaces.label(row.modelData); color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 13; elide: Text.ElideRight }
                    Icon { name: row.index === 0 ? "code" : row.index === 1 ? "globe" : row.index === 2 ? "package-x-generic" : "preferences-system"; size: 17; color: Theme.inkDim }
                    Text { text: root.workspaces.countFor(row.modelData); color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 10 }
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.workspaces.focus(row.modelData) }
            }
        }
        Text { visible: !root.rows.length; text: "No workspaces available"; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 13 }
    }
}
