import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var notice
    property bool popupView: false
    signal request(var action)
    implicitHeight: content.implicitHeight + 30
    radius: 19
    function send(action, extra) {
        root.request(Object.assign({action:action,"id":root.notice.id,revision:root.notice.revision}, extra || {}));
    }
    ColumnLayout {
        id: content
        anchors.fill: parent; anchors.margins: 15; spacing: 9
        RowLayout {
            Layout.fillWidth: true; spacing: 8
            Icon { name: "bell"; size: 14; color: root.notice.urgency === 2 ? Theme.rose : Theme.cyan }
            Text { Layout.fillWidth: true; text: root.notice.app_name || "Application"; textFormat: Text.PlainText; font.family: Theme.fontSans; font.pixelSize: 10; color: Theme.inkDim; elide: Text.ElideRight }
            Text { visible: root.notice.urgency === 2 || !root.notice.read; text: root.notice.urgency === 2 ? "CRITICAL" : "NEW"; color: root.notice.urgency === 2 ? Theme.rose : Theme.mint; font.family: Theme.fontMono; font.pixelSize: 8 }
            IconTile {
                objectName: (root.popupView ? "dismissPopup" : "removeNotice") + root.notice.id
                iconName: "close"; size: 25; iconSize: 12
                Accessible.name: root.popupView ? "Dismiss notification" : "Remove notification"
                onClicked: root.send(root.popupView ? "dismiss" : "remove")
            }
        }
        Text { Layout.fillWidth: true; text: root.notice.summary; textFormat: Text.PlainText; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 14; font.weight: Font.DemiBold; wrapMode: Text.Wrap; maximumLineCount: 2; elide: Text.ElideRight }
        Text { Layout.fillWidth: true; visible: text.length > 0; text: root.notice.body; textFormat: Text.PlainText; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11; lineHeight: 1.25; wrapMode: Text.Wrap; maximumLineCount: root.popupView ? 3 : 8; elide: Text.ElideRight }
        Flow {
            Layout.fillWidth: true
            spacing: 6
            visible: root.notice.active && root.notice.actions.length > 0
            Repeater {
                model: root.notice.active ? root.notice.actions : []
                NoticeAction {
                    required property var modelData
                    required property int index
                    objectName: "noticeAction" + root.notice.id + "_" + index
                    text: modelData.label
                    width: Math.min(implicitWidth, content.width)
                    onClicked: root.send("invoke", {key:modelData.key})
                }
            }
        }
    }
}
