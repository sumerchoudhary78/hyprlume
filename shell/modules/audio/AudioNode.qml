import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var controller
    required property var node
    required property int nodeIndex
    property string prefix: "output"
    readonly property bool selected: node === (node.isSink ? controller.defaultSink : controller.defaultSource)
    readonly property bool usable: controller.available && node.ready && !!node.audio
    implicitHeight: 91; radius: 15
    color: selected ? "#48223d57" : "#290c192a"
    border.width: 1; border.color: selected ? "#5577cde0" : Theme.rim
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 12; spacing: 2
        RowLayout {
            Layout.fillWidth: true; spacing: 9
            Icon { name: root.node.isStream ? "play" : root.node.isSink ? "audio-volume-high" : "audio-input-microphone"; size: 17; color: root.selected ? Theme.cyan : Theme.inkDim }
            Text {
                Layout.fillWidth: true; text: root.controller.label(root.node)
                textFormat: Text.PlainText; elide: Text.ElideRight
                font.family: Theme.fontSans; font.pixelSize: 12; color: Theme.ink
            }
            GlassButton {
                objectName: root.prefix + "Default" + root.nodeIndex
                visible: !root.node.isStream
                text: root.selected ? "Default" : "Use"; compact: true
                enabled: root.usable && !root.selected
                onClicked: root.controller.setDefault(root.node)
            }
        }
        RowLayout {
            Layout.fillWidth: true; spacing: 8
            GlassButton {
                objectName: root.prefix + "Mute" + root.nodeIndex
                compact: true; text: root.usable && root.node.audio.muted ? "Unmute" : "Mute"
                enabled: root.usable; Accessible.name: text + " " + root.controller.label(root.node)
                onClicked: root.controller.toggleMute(root.node)
            }
            GlassSlider {
                objectName: root.prefix + "Volume" + root.nodeIndex
                Layout.fillWidth: true; enabled: root.usable
                value: root.usable && Number.isFinite(root.node.audio.volume) ? root.node.audio.volume : 0
                Accessible.name: root.controller.label(root.node) + " volume"
                onMoved: root.controller.setVolume(root.node, value)
            }
            Text {
                Layout.preferredWidth: 37; horizontalAlignment: Text.AlignRight
                text: root.usable ? Math.round(root.node.audio.volume * 100) + "%" : "—"
                color: Theme.inkDim; font.family: Theme.fontMono; font.pixelSize: 11
            }
        }
    }
}
