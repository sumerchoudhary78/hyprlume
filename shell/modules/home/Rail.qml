import QtQuick
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var workspaces
    property var backend
    radius: 35
    secondaryColor: Theme.amber
    glowColor: Theme.amber
    tintTop: "#be251b29"
    tint: "#d0101423"
    Column {
        anchors.horizontalCenter: parent.horizontalCenter; y: 35; spacing: 17
        Repeater {
            model: ["user-home","view-grid","utilities-terminal","package-x-generic","cloud","layers","preferences-system"]
            IconTile {
                required property string modelData
                required property int index
                iconName: modelData; size: 48; iconSize: 23
                active: index === 0
                accent: Theme.amber
                onClicked: {
                    if (index === 0 && root.workspaces.all.length) root.workspaces.focus(root.workspaces.all[0]);
                    else if (index === 2 && root.backend) root.backend.send({op:"command",text:"launch terminal",dry_run:true});
                    else if (index === 6 && root.backend) root.backend.send({op:"toggle"});
                    else if (root.workspaces.all.length) root.workspaces.focus(root.workspaces.all[index % root.workspaces.all.length]);
                }
            }
        }
    }
}
