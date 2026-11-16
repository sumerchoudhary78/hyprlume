import QtQuick
import qs.components
import qs.modules.scene
import qs.theme

// Isolated visual fixture, not an alternate implementation of the effects.
Item {
    id: root
    property bool finished: false
    property int stage: 0
    property int savedTicks: 0
    readonly property var samples: [
        {fraction:0,available:true,label:"EMPTY"},
        {fraction:0.18,available:true,label:"18% VOLUME"},
        {fraction:0.50,available:true,label:"HALF FULL"},
        {fraction:1,available:true,label:"FULL"},
        {fraction:0.65,available:false,label:"UNAVAILABLE"},
        {fraction:NaN,available:true,label:"INVALID INPUT"}
    ]
    Wallpaper { id: wallpaper; anchors.fill: parent }
    GlassSurface {
        id: plate
        x: 30; y: 65; width: parent.width - 60; height: parent.height - 100
        radius: 34; backdrop: wallpaper
        tint: "#d4061021"
    }
    Text {
        x: 52; y: 25
        text: "LIQUID MATERIAL / NATIVE SHADER CHECK"
        color: Theme.ink; font.family: Theme.fontMono; font.pixelSize: 14; font.letterSpacing: 1.5
    }
    Row {
        id: row
        anchors.horizontalCenter: parent.horizontalCenter
        y: 130
        spacing: 24
        Repeater {
            id: gauges
            model: root.samples
            delegate: Column {
                required property var modelData
                spacing: 24
                Gauge {
                    id: gauge
                    width: 150; height: 150
                    value: modelData.fraction; available: modelData.available
                    accent: Theme.mint
                    showLabels: false
                    phaseOffset: 1.1
                }
                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: modelData.label
                    color: Theme.ink; font.family: Theme.fontMono; font.pixelSize: 11
                }
                property alias meter: gauge
            }
        }
    }
    Text {
        x: 52; anchors.bottom: parent.bottom; anchors.bottomMargin: 49
        text: "Area tracks the metric. Unknown readings stay dry. Motion pauses when hidden."
        color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 13
    }
    LiquidMotion { id: clock; settleDuration: 120; hovered: true }
    Timer {
        id: verify
        interval: 170; repeat: true; running: true
        onTriggered: {
            const fail = message => { console.error("HYPRLUME_EFFECTS_FAIL " + message); Qt.exit(1); };
            if (root.stage === 0) {
                if (clock.tickCount < 1) return;
                if (!clock.running) { fail("clock did not start"); return; }
                clock.active = false; root.savedTicks = clock.tickCount;
            } else if (root.stage === 1) {
                if (clock.running || clock.tickCount !== root.savedTicks) { fail("hidden clock kept ticking"); return; }
                clock.reducedMotion = true; clock.active = true; clock.pulse();
            } else if (root.stage === 2) {
                if (clock.running || clock.tickCount !== root.savedTicks) { fail("reduced-motion clock kept ticking"); return; }
                clock.reducedMotion = false;
            } else if (root.stage === 3) {
                if (clock.tickCount <= root.savedTicks) return;
                if (!clock.running) { fail("clock did not resume"); return; }
                clock.hovered = false; clock.pulse();
            } else if (root.stage === 4) {
                if (clock.running) { fail("clock did not settle"); return; }
                const regions = [];
                for (let i = 0; i < gauges.count; i++) {
                    const meter = gauges.itemAt(i).meter;
                    const point = meter.mapToItem(root, meter.width/2, meter.height/2);
                    regions.push({x:point.x,y:point.y,radius:meter.width*0.43,fraction:meter.hasValue ? meter.fraction : 0,available:meter.hasValue});
                }
                root.finished = true;
                console.log("HYPRLUME_EFFECTS_RESULT " + JSON.stringify({motion:"hidden/reduced/resume/settle passed",regions:regions}));
                verify.stop(); clock.active = false;
            }
            root.stage++;
        }
    }
}
