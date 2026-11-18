import QtQuick
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var settings
    required property string selected
    signal selectedOutput(string name)

    readonly property var rectangles: settings.map(setting => {
        const resolution = setting.mode.split("@")[0].split("x");
        const scale = setting.scale > 0 ? setting.scale : 1;
        return {name: setting.name, x: setting.x, y: setting.y,
            width: Number(resolution[0]) / scale, height: Number(resolution[1]) / scale};
    }).filter(rect => Number.isFinite(rect.x + rect.y + rect.width + rect.height))
    readonly property real minimumX: Math.min(0, ...rectangles.map(rect => rect.x))
    readonly property real minimumY: Math.min(0, ...rectangles.map(rect => rect.y))
    readonly property real spanX: Math.max(1, ...rectangles.map(rect => rect.x + rect.width)) - minimumX
    readonly property real spanY: Math.max(1, ...rectangles.map(rect => rect.y + rect.height)) - minimumY
    readonly property real zoom: Math.max(0, Math.min((width-26)/spanX, (height-24)/spanY))

    radius: 13
    color: "#90102034"
    border.color: Theme.rim
    clip: true
    Repeater {
        model: root.rectangles
        Rectangle {
            required property var modelData
            x: (root.width - root.spanX*root.zoom)/2 + (modelData.x-root.minimumX)*root.zoom
            y: (root.height - root.spanY*root.zoom)/2 + (modelData.y-root.minimumY)*root.zoom
            width: Math.max(12, modelData.width*root.zoom)
            height: Math.max(10, modelData.height*root.zoom)
            radius: 6
            color: modelData.name === root.selected ? "#6065c8e0" : "#60486587"
            border.color: modelData.name === root.selected ? Theme.cyan : Theme.rimStrong
            Text {
                anchors.centerIn: parent
                width: parent.width - 4
                text: parent.modelData.name
                textFormat: Text.PlainText
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
                color: Theme.ink
                font.family: Theme.fontMono
                font.pixelSize: 11
            }
            TapHandler { onTapped: root.selectedOutput(parent.modelData.name) }
        }
    }
}
