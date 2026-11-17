import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var controller
    readonly property var player: controller.player
    implicitHeight: content.implicitHeight + 30
    radius: 20; color: "#6815223d"; border.width: 1; border.color: "#587d78b0"
    ColumnLayout {
        id: content
        anchors.fill: parent; anchors.margins: 15; spacing: 10
        RowLayout {
            Layout.fillWidth: true
            Icon { name: "play"; size: 21; color: Theme.violet }
            Text { Layout.fillWidth: true; text: "Now playing"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 15; font.weight: Font.DemiBold }
            StatusDot { color: root.player && root.player.isPlaying ? Theme.cyan : Theme.inkFaint }
        }
        Flow {
            Layout.fillWidth: true; spacing: 6
            Repeater {
                model: root.controller.players
                GlassButton {
                    required property var modelData
                    required property int index
                    objectName: "mediaPlayer" + index
                    compact: true; primary: modelData === root.player
                    text: root.controller.displayText(modelData.identity, "Player", 20)
                    onClicked: root.controller.selectPlayer(modelData)
                }
            }
        }
        Text {
            Layout.fillWidth: true
            text: root.player ? root.controller.displayText(root.player.trackTitle, "Untitled track", 300) : "No active media player"
            textFormat: Text.PlainText; elide: Text.ElideRight
            font.family: Theme.fontSans; font.pixelSize: 20; font.weight: Font.DemiBold; color: Theme.ink
        }
        Text {
            Layout.fillWidth: true
            text: root.player ? root.controller.displayText([root.player.trackArtist, root.player.trackAlbum].filter(Boolean).join(" · "), "", 300) : "Start playback in an MPRIS-compatible app."
            textFormat: Text.PlainText; elide: Text.ElideRight
            font.family: Theme.fontSans; font.pixelSize: 11; color: Theme.inkDim
        }
        RowLayout {
            Layout.fillWidth: true; spacing: 9
            GlassButton { objectName: "mediaPrevious"; text: "Previous"; compact: true; enabled: !!root.player && root.player.canControl && root.player.canGoPrevious; onClicked: root.controller.playback(root.player, "previous") }
            GlassButton { objectName: "mediaToggle"; text: root.player && root.player.isPlaying ? "Pause" : "Play"; primary: true; Layout.fillWidth: true; enabled: !!root.player && root.player.canControl && root.player.canTogglePlaying; onClicked: root.controller.playback(root.player, "toggle") }
            GlassButton { objectName: "mediaNext"; text: "Next"; compact: true; enabled: !!root.player && root.player.canControl && root.player.canGoNext; onClicked: root.controller.playback(root.player, "next") }
        }
    }
}
