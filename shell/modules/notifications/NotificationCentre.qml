import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    readonly property var snapshot: backend.notifications
    radius: Theme.radiusPanel
    focus: true
    function send(request) { backend.send({op:"notifications",request:request}); }
    Keys.onEscapePressed: root.send({action:"close"})
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 20; spacing: 17
        RowLayout {
            Layout.fillWidth: true; spacing: 10
            Icon { name: "bell"; color: Theme.cyan; size: 22 }
            Text { Layout.fillWidth: true; text: "Notifications"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 23; font.weight: Font.DemiBold }
            IconTile { objectName: "closeNotifications"; iconName: "close"; size: 30; Accessible.name: "Close notification centre"; onClicked: root.send({action:"close"}) }
        }
        RowLayout {
            Layout.fillWidth: true
            GlassCheckBox {
                objectName: "notificationDnd"
                text: "Do Not Disturb"; checked: root.snapshot.dnd
                onClicked: root.send({action:"dnd",enabled:checked})
            }
            Item { Layout.fillWidth: true }
            GlassButton { objectName: "clearNotifications"; text: "Clear all"; compact: true; enabled: root.snapshot.items.length > 0; onClicked: root.send({action:"clear"}) }
        }
        Text { visible: root.snapshot.dnd; Layout.fillWidth: true; text: "Popups paused. New alerts stay in your history."; font.family: Theme.fontSans; font.pixelSize: 10; color: Theme.violet; wrapMode: Text.Wrap }
        ListView {
            id: list
            Layout.fillWidth: true; Layout.fillHeight: true
            clip: true; spacing: 12
            model: root.snapshot.items
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: NoticeCard {
                required property var modelData
                width: list.width; height: implicitHeight
                notice: modelData; reducedEffects: root.reducedEffects
                onRequest: action => root.send(action)
            }
            Column {
                anchors.centerIn: parent; spacing: 12
                visible: list.count === 0
                Icon { anchors.horizontalCenter: parent.horizontalCenter; name: "bell"; size: 35; color: Theme.inkFaint }
                Text { text: "All caught up"; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 16 }
            }
        }
        Text {
            Layout.fillWidth: true
            text: root.backend.error || root.snapshot.error || (root.snapshot.available ? "History stays in memory for this session." : "Notification service unavailable")
            textFormat: Text.PlainText; wrapMode: Text.Wrap
            color: root.snapshot.error || root.backend.error ? Theme.rose : Theme.inkFaint
            font.family: Theme.fontSans; font.pixelSize: 10
        }
    }
    Component.onCompleted: forceActiveFocus()
}
