import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

Button {
    id: root
    required property var controller
    required property var entry
    implicitHeight: entry.isSeparator ? 10 : 40
    enabled: !entry.isSeparator && entry.enabled
    Accessible.name: controller.label(entry.text)
    opacity: entry.enabled || entry.isSeparator ? 1 : 0.4
    Keys.onReturnPressed: event => { if (!event.isAutoRepeat && root.enabled) root.clicked(); }
    Keys.onEnterPressed: event => { if (!event.isAutoRepeat && root.enabled) root.clicked(); }
    background: Rectangle {
        anchors.fill: parent; radius: 12
        color: root.hovered || root.activeFocus ? "#503b5a7b" : "#24192a43"
        border.width: root.activeFocus ? 1 : 0; border.color: Theme.cyan
        visible: !root.entry.isSeparator
    }
    contentItem: Item {
        Rectangle { anchors.verticalCenter: parent.verticalCenter; width: parent.width; height: 1; color: Theme.divider; visible: root.entry.isSeparator }
        RowLayout {
            anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 10; spacing: 9
            visible: !root.entry.isSeparator
            Text { Layout.preferredWidth: 17; text: root.entry.buttonType === 2 ? (root.entry.checkState === Qt.Checked ? "●" : "○") : root.entry.buttonType ? (root.entry.checkState === Qt.Checked ? "✓" : "□") : ""; color: Theme.cyan; font.pixelSize: 13 }
            Text { Layout.fillWidth: true; text: root.controller.label(root.entry.text); textFormat: Text.PlainText; elide: Text.ElideRight; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 12 }
            Icon { visible: root.entry.hasChildren; name: "chevron-right"; size: 14; color: Theme.inkDim }
        }
    }
    onClicked: controller.trigger(entry)
}
