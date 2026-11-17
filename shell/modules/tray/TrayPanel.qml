import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    readonly property var tray: backend.tray
    radius: Theme.radiusPanel; tintTop: "#f01a2b45"; tint: "#f0081225"
    glowColor: Theme.cyan; secondaryColor: Theme.violet
    focus: true
    function close() { backend.send({op:"panel",panel:"tray",open:false}); }
    Keys.onEscapePressed: { if (tray.menuStack.length > 1) { tray.back(); Qt.callLater(() => root.forceActiveFocus()); } else close(); }
    Keys.onDownPressed: menu.step(1)
    Keys.onUpPressed: menu.step(-1)
    Keys.onLeftPressed: tray.back()
    Keys.onPressed: event => {
        if (event.key === Qt.Key_Home) { menu.currentIndex = -1; menu.step(1); event.accepted = true; }
        else if (event.key === Qt.Key_End) { menu.currentIndex = 0; menu.step(-1); event.accepted = true; }
    }
    Component.onCompleted: { forceActiveFocus(); if (!tray.selected && tray.items.length) tray.open(tray.items[0]); }
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 20; spacing: 15
        RowLayout {
            Layout.fillWidth: true
            Icon { name: "grid"; size: 23; color: Theme.cyan }
            Text { Layout.fillWidth: true; text: "Application tray"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 23; font.weight: Font.DemiBold }
            IconTile { objectName: "closeTray"; iconName: "close"; size: 30; Accessible.name: "Close application tray"; onClicked: root.close() }
        }
        ListView {
            id: apps
            Layout.fillWidth: true; Layout.preferredHeight: Math.min(180, contentHeight)
            clip: true; spacing: 7; model: root.tray.items
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Rectangle {
                id: app
                required property var modelData
                required property int index
                width: apps.width; height: 50; radius: 14
                color: root.tray.selected === modelData ? "#50314d6e" : "#34132136"; border.width: 1; border.color: Theme.rim
                RowLayout {
                    anchors.fill: parent; anchors.margins: 8; spacing: 10
                    TrayIcon { objectName: "trayIcon" + app.index; controller: root.tray; trayItem: app.modelData; onActivated: root.close() }
                    Text { Layout.fillWidth: true; text: root.tray.label(app.modelData.title || app.modelData.id, "Application"); textFormat: Text.PlainText; elide: Text.ElideRight; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 12 }
                    GlassButton { objectName: "trayOpen" + app.index; text: "Open"; compact: true; visible: !app.modelData.onlyMenu; onClicked: { if (root.tray.activate(app.modelData, false)) root.close(); } }
                    GlassButton { objectName: "trayMenu" + app.index; text: "Menu"; compact: true; enabled: app.modelData.hasMenu; onClicked: root.tray.open(app.modelData) }
                }
            }
        }
        Text { visible: root.tray.items.length === 0; text: "No active tray applications"; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 14 }
        RowLayout {
            Layout.fillWidth: true
            GlassButton { objectName: "trayBack"; text: "Back"; compact: true; visible: root.tray.menuStack.length > 1; onClicked: root.tray.back() }
            Text { Layout.fillWidth: true; text: root.tray.menuTitle; textFormat: Text.PlainText; elide: Text.ElideRight; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 13; font.weight: Font.DemiBold }
        }
        ListView {
            id: menu
            Layout.fillWidth: true; Layout.fillHeight: true; model: root.tray.visibleEntries
            clip: true; spacing: 5
            keyNavigationEnabled: false
            function step(delta) {
                for (let i = 1; i <= count; i++) {
                    const base = currentIndex < 0 ? (delta > 0 ? -1 : 0) : currentIndex;
                    const next = (base + delta * i + count * 2) % count;
                    const entry = root.tray.visibleEntries[next];
                    if (!entry.isSeparator && entry.enabled) {
                        currentIndex = next; positionViewAtIndex(next, ListView.Contain);
                        Qt.callLater(() => { const item = itemAtIndex(next); if (item) item.forceActiveFocus(); });
                        return;
                    }
                }
            }
            onModelChanged: { currentIndex = -1; root.forceActiveFocus(); }
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: TrayMenuRow { required property var modelData; required property int index; objectName: "trayMenuEntry" + index; width: menu.width; controller: root.tray; entry: modelData }
            Text { anchors.centerIn: parent; visible: menu.count === 0; text: root.tray.selected ? "No menu actions available" : "Choose an application"; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 12 }
        }
        Text { Layout.fillWidth: true; wrapMode: Text.Wrap; text: root.tray.error || "Icons: left click to open, middle click for secondary action, right click for menu. Scroll is forwarded to the app."; textFormat: Text.PlainText; color: root.tray.error ? Theme.amber : Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 10 }
    }
}
