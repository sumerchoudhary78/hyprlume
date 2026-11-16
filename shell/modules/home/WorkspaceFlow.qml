import QtQuick
import qs.components
import qs.theme

// The three real stages of every desktop action: intent, review, execution.
GlassSurface {
    id: root
    required property var backend
    property var workspaces
    secondaryColor: Theme.amber
    glowColor: Theme.violet
    readonly property bool review: backend.snapshot.confirmation !== null && backend.snapshot.confirmation !== undefined
    PanelHeader { x: 17; y: 16; width: parent.width - 34; title: "Command Graph"; iconName: "workflow"; accent: Theme.violet }
    Canvas {
        x: 15; y: 42; width: parent.width - 30; height: parent.height - 48
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        onPaint: {
            const c=getContext("2d");c.clearRect(0,0,width,height);
            c.strokeStyle="#5ccedb";c.lineWidth=1.0;
            c.beginPath();c.moveTo(width*0.5,39);c.bezierCurveTo(width*0.5,77,width*0.23,62,width*0.23,height-30);c.stroke();
            c.beginPath();c.moveTo(width*0.5,39);c.bezierCurveTo(width*0.5,77,width*0.79,62,width*0.79,height-30);c.stroke();
            c.fillStyle="#89ecff";c.fillRect(width*0.5-2,41,4,4);
        }
    }
    GraphNode {
        x: (parent.width-width)/2; y: 52; label: "intent"; accent: Theme.cyan; iconName: "check"
        onClicked: root.backend.send({op:"toggle"})
    }
    GraphNode {
        x: 28; y: parent.height - 48; label: "review"; accent: Theme.violet; iconName: "check"
        onClicked: root.backend.send({op:"toggle"})
    }
    GraphNode {
        x: parent.width-width-25; y: parent.height - 48; label: "execute"; accent: Theme.amber; iconName: "play"
        enabled: root.review
        onClicked: root.backend.send({op:"confirm",token:root.backend.snapshot.confirmation})
    }
    component GraphNode: Item {
        id: node
        property string label
        property color accent
        property string iconName
        signal clicked()
        width: 91; height: 30; opacity: enabled ? 1 : 0.55
        Rectangle { anchors.fill: parent; radius: 13; color: Qt.rgba(node.accent.r*0.22,node.accent.g*0.22,node.accent.b*0.22,0.9); border.width: 1; border.color: node.accent }
        Row { anchors.centerIn: parent; spacing: 10
            Text { text: node.label; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 12 }
            Icon { name: node.iconName; color: node.accent; size: 12 }
        }
        MouseArea { anchors.fill: parent; enabled: node.enabled; cursorShape: Qt.PointingHandCursor; onClicked: node.clicked() }
    }
}
