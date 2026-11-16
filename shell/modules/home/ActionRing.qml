import QtQuick
import qs.components
import qs.theme

// A perspective deck with live action satellites. It has no frame timer.
Item {
    id: root
    required property var backend
    required property var workspaces
    property bool reducedEffects: false
    readonly property var actions: [
        {label:"Terminal",icon:"code",x:15,y:45,op:"terminal"},
        {label:"Command",icon:"utilities-terminal",x:180,y:5,op:"command"},
        {label:"Explore",icon:"system-search",x:353,y:45,op:"explore"},
        {label:"Voice",icon:"audio-input-microphone",x:0,y:137,op:"voice"},
        {label:"Workspace",icon:"workflow",x:369,y:137,op:"workspace"}
    ]
    Canvas {
        anchors.fill: parent
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        onPaint: {
            const c=getContext("2d"),cx=width/2,cy=height*0.67;
            c.clearRect(0,0,width,height);
            const g=c.createRadialGradient(cx,cy,0,cx,cy,width*0.46);
            g.addColorStop(0,"rgba(71,80,210,0.52)");g.addColorStop(0.55,"rgba(56,43,144,0.19)");g.addColorStop(1,"rgba(0,5,18,0)");
            c.fillStyle=g;c.fillRect(0,0,width,height);
            for(const ring of [[119,48,"#9e7cff"],[130,53,"#664cb7"],[143,60,"#69dfff"],[155,65,"#3d558e"],[226,91,"#6671ad"]]) {
                c.beginPath();c.ellipse(cx-ring[0],cy-ring[1],ring[0]*2,ring[1]*2);
                c.strokeStyle=ring[2];c.lineWidth=1;c.globalAlpha=0.8;c.stroke();
            }
            c.globalAlpha=1;
            for(let i=0;i<72;i++) {
                const a=i/72*Math.PI*2;
                c.beginPath();c.moveTo(cx+Math.cos(a)*135,cy+Math.sin(a)*56);c.lineTo(cx+Math.cos(a)*143,cy+Math.sin(a)*61);
                c.strokeStyle=i%5===0?"#bae8ff":"#665ac0";c.lineWidth=i%5===0?1.5:0.7;c.stroke();
            }
            for(const arc of [[Math.PI*0.05,Math.PI*0.49,"#ffbd7a"],[Math.PI*1.06,Math.PI*1.55,"#73eaff"],[Math.PI*0.57,Math.PI*0.95,"#ae85ff"]]) {
                c.save();c.translate(cx,cy);c.scale(1,0.4);c.beginPath();c.arc(0,0,143,arc[0],arc[1]);c.strokeStyle=arc[2];c.lineWidth=3;c.stroke();c.restore();
            }
        }
    }
    Repeater {
        model: root.actions
        delegate: GlassSurface {
            id: tile
            objectName: "launcher" + modelData.op
            required property var modelData
            width: 126; height: 58; x: modelData.x; y: modelData.y
            radius: 16
            reducedEffects: root.reducedEffects
            glowColor: modelData.op === "command" ? Theme.cyan : Theme.violet
            secondaryColor: glowColor
            tint: "#a3181a3d"; tintTop: "#99343b64"
            rotation: modelData.x < 80 ? -7 : modelData.x > 300 ? 7 : -2
            transform: Rotation { origin.x: 63; origin.y: 29; axis.x: 1; axis.y: 0; angle: 10 }
            Column {
                anchors.centerIn: parent; spacing: 3
                Icon { anchors.horizontalCenter: parent.horizontalCenter; name: tile.modelData.icon; size: 21; color: Theme.ink }
                Text { anchors.horizontalCenter: parent.horizontalCenter; text: tile.modelData.label; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 12 }
            }
            MouseArea {
                anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                onEntered: tile.scale = 1.04
                onExited: tile.scale = 1
                onClicked: {
                    const op=tile.modelData.op;
                    if(op === "voice") root.backend.send({op:"voice",mode:root.backend.snapshot.voice === "off" ? "wake" : "off"});
                    else if(op === "workspace") root.workspaces.focusNext();
                    else if(op === "terminal") root.backend.send({op:"command",text:"launch terminal",dry_run:true});
                    else root.backend.send({op:"toggle"});
                }
            }
            Behavior on scale { NumberAnimation { duration: 130; easing.type: Easing.OutCubic } }
        }
    }
    Item {
        x: (parent.width-width)/2; y: parent.height*0.67-height/2; width: 100; height: 66
        Rectangle {
            anchors.fill: parent; radius: 44; rotation: -5
            color: "#4c19347b"; border.width: 1; border.color: "#a47ce7ff"
        }
        Icon { anchors.centerIn: parent; name: "bolt"; size: 44; color: Theme.violet }
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.backend.send({op:"toggle"}) }
    }
}
