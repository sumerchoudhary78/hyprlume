import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    required property var workspaces
    secondaryColor: Theme.amber
    glowColor: Theme.violet
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 17; spacing: 6
        PanelHeader {
            Layout.fillWidth: true; title: "Quick Actions"; iconName: "bolt"; accent: Theme.amber
            Text { text: "5 actions"; color: Theme.inkFaint; font.pixelSize: 11 }
        }
        Item { Layout.preferredHeight: 2 }
        ActionRow { Layout.fillWidth: true; objectName: "quickCommand"; label: "Run command"; iconName: "utilities-terminal"; onClicked: root.backend.send({op:"toggle"}) }
        ActionRow { Layout.fillWidth: true; objectName: "quickTerminal"; label: "Open terminal"; iconName: "text-x-script"; onClicked: root.backend.send({op:"command", text:"launch terminal", dry_run:true}) }
        ActionRow { Layout.fillWidth: true; objectName: "nextWorkspace"; label: "Next workspace"; iconName: "workflow"; onClicked: root.workspaces.focusNext() }
        ActionRow { Layout.fillWidth: true; objectName: "toggleVoice"; label: root.backend.snapshot.voice === "off" ? "Enable wake word" : "Disable microphone"; iconName: "audio-input-microphone"; onClicked: root.backend.send({op:"voice", mode:root.backend.snapshot.voice === "off" ? "wake" : "off"}) }
        ActionRow { Layout.fillWidth: true; objectName: "dictateNote"; label: "Dictate a note"; iconName: "cloud"; onClicked: root.backend.send({op:"voice", mode:"dictate"}) }
    }
}
