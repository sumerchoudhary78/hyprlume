import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    readonly property var audio: backend.audio
    radius: Theme.radiusPanel
    tintTop: "#ef1a2c47"; tint: "#f0081225"
    glowColor: Theme.cyan; secondaryColor: Theme.violet
    focus: true
    function close() { backend.send({op:"panel",panel:"audio",open:false}); }
    Keys.onEscapePressed: root.close()
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 20; spacing: 15
        RowLayout {
            Layout.fillWidth: true
            Layout.minimumWidth: root.width - 40
            Icon { name: "audio-volume-high"; size: 23; color: Theme.cyan }
            ColumnLayout {
                Layout.fillWidth: true; spacing: 2
                Text { text: "Sound & media"; font.family: Theme.fontSans; font.pixelSize: 23; font.weight: Font.DemiBold; color: Theme.ink }
                Text { text: "Your devices. Your atmosphere."; font.family: Theme.fontSans; font.pixelSize: 10; color: Theme.inkDim }
            }
            IconTile { objectName: "closeAudio"; iconName: "close"; size: 30; Accessible.name: "Close sound and media"; onClicked: root.close() }
        }
        ScrollView {
            id: scroll
            Layout.fillWidth: true; Layout.fillHeight: true
            clip: true; contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                width: scroll.availableWidth; spacing: 14
                RowLayout {
                    Layout.fillWidth: true; spacing: 15
                    Gauge {
                        Layout.preferredWidth: 90; Layout.preferredHeight: 90
                        label: "OUTPUT"; available: root.audio.available && !!root.audio.defaultSink && root.audio.defaultSink.ready
                        value: available ? root.audio.defaultSink.audio.volume : 0
                        valueText: available && root.audio.defaultSink.audio.muted ? "MUTED" : hasValue ? Math.round(value * 100) + "%" : "—"
                        reducedEffects: root.reducedEffects
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Text { Layout.fillWidth: true; text: root.audio.label(root.audio.defaultSink); textFormat: Text.PlainText; wrapMode: Text.Wrap; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 14 }
                        Text { Layout.fillWidth: true; text: root.audio.available ? "Live PipeWire controls" : "PipeWire is unavailable"; color: root.audio.available ? Theme.cyan : Theme.amber; font.family: Theme.fontSans; font.pixelSize: 11 }
                    }
                }
                MediaCard { Layout.fillWidth: true; controller: root.audio }
                Repeater {
                    model: [
                        {title:"Output devices",prefix:"output",nodes:root.audio.outputs},
                        {title:"Input devices",prefix:"input",nodes:root.audio.inputs},
                        {title:"Application streams",prefix:"stream",nodes:root.audio.streams}
                    ]
                    ColumnLayout {
                        id: section
                        required property var modelData
                        Layout.fillWidth: true; spacing: 8
                        Text { text: section.modelData.title; font.family: Theme.fontSans; font.pixelSize: 12; font.weight: Font.DemiBold; color: Theme.inkDim }
                        Repeater {
                            model: section.modelData.nodes
                            AudioNode { required property var modelData; required property int index; Layout.fillWidth: true; node: modelData; nodeIndex: index; prefix: section.modelData.prefix; controller: root.audio }
                        }
                        Text { visible: section.modelData.nodes.length === 0; text: root.audio.available ? "No devices or streams in this section" : "Waiting for the audio server"; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11 }
                    }
                }
            }
        }
        Text {
            Layout.fillWidth: true; wrapMode: Text.Wrap
            text: root.audio.error || "Volumes are limited to 100%. Device changes follow your session's routing policy."
            color: root.audio.error ? Theme.amber : Theme.inkFaint
            font.family: Theme.fontSans; font.pixelSize: 10
        }
    }
}
