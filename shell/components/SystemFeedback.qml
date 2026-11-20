import QtQuick
import QtQuick.Layouts
import qs.theme

GlassSurface {
    id: root
    required property var feedback
    radius: 20
    tintTop: "#f0213350"
    tint: "#ed0a1529"

    RowLayout {
        anchors.fill: parent
        anchors.margins: 13
        spacing: 12
        Icon { name: root.feedback?.icon || "keyboard"; size: 26; color: Theme.cyan }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            RowLayout {
                Layout.fillWidth: true
                Text {
                    Layout.fillWidth: true
                    text: root.feedback?.title || ""
                    textFormat: Text.PlainText
                    color: Theme.ink
                    font.family: Theme.fontSans
                    font.pixelSize: 14
                }
                Text {
                    Layout.maximumWidth: 175
                    text: root.feedback?.detail || ""
                    textFormat: Text.PlainText
                    elide: Text.ElideRight
                    color: Theme.inkDim
                    font.family: Theme.fontSans
                    font.pixelSize: 12
                }
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 4
                visible: root.feedback?.percent !== null && root.feedback?.percent !== undefined
                radius: 2
                color: "#4a7199ac"
                Rectangle {
                    width: parent.width * (root.feedback?.percent || 0) / 100
                    height: parent.height
                    radius: 2
                    color: Theme.cyan
                }
            }
        }
    }
}
