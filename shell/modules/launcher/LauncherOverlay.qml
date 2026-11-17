import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.components
import qs.theme

Item {
    id: root
    required property var backend
    property bool reducedEffects: false
    readonly property var controller: backend.launcher
    function close() {
        const panel = backend.snapshot.desktop_panel;
        if (panel === "launcher" || panel === "switcher") backend.send({op:"panel",panel:panel,open:false});
    }
    Rectangle { anchors.fill: parent; color: "#98050b18"; MouseArea { anchors.fill: parent; onClicked: root.close() } }
    Connections { target: root.controller; function onActivated() { root.close(); } }
    Binding { target: root.controller; property: "mode"; value: root.backend.snapshot.desktop_panel === "switcher" ? "windows" : "apps" }
    GlassSurface {
        anchors.centerIn: parent; width: Math.min(680, parent.width - 32); height: Math.min(630, parent.height - 32)
        radius: Theme.radiusPanel; glowColor: Theme.cyan; secondaryColor: Theme.violet; reducedEffects: root.reducedEffects
        tintTop: "#f01a2b45"; tint: "#f0081225"
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        ColumnLayout {
            anchors.fill: parent; anchors.margins: 22; spacing: 14
            RowLayout {
                Layout.fillWidth: true
                Icon { name: "system-search"; color: Theme.cyan; size: 24 }
                Text { Layout.fillWidth: true; text: "At your fingertips"; font.family: Theme.fontSans; font.pixelSize: 24; color: Theme.ink }
                IconTile { objectName: "closeLauncher"; iconName: "close"; size: 30; onClicked: root.close() }
            }
            RowLayout {
                GlassButton { objectName: "launcherApps"; text: "Applications"; primary: root.controller.mode === "apps"; onClicked: root.backend.send({op:"panel",panel:"launcher",open:true}) }
                GlassButton { objectName: "launcherWindows"; text: "Windows"; primary: root.controller.mode === "windows"; onClicked: root.backend.send({op:"panel",panel:"switcher",open:true}) }
                Item { Layout.fillWidth: true }
            }
            TextField {
                id: search
                objectName: "launcherSearch"
                Layout.fillWidth: true; Layout.preferredHeight: 48
                placeholderText: root.controller.mode === "apps" ? "Search apps, names and keywords…" : "Find an open window…"
                text: root.controller.query; maximumLength: 128; color: Theme.ink; placeholderTextColor: Theme.inkFaint
                font.family: Theme.fontSans; font.pixelSize: 16; selectByMouse: true
                background: Rectangle { radius: 15; color: "#80102136"; border.color: search.activeFocus ? Theme.cyan : Theme.rim }
                onTextEdited: root.controller.query = text
                Keys.onDownPressed: list.currentIndex = Math.min(list.count-1, list.currentIndex+1)
                Keys.onUpPressed: list.currentIndex = Math.max(0, list.currentIndex-1)
                Keys.onEscapePressed: root.close()
                onAccepted: root.controller.activate(root.controller.results[list.currentIndex])
                Component.onCompleted: forceActiveFocus()
            }
            ListView {
                id: list
                Layout.fillWidth: true; Layout.fillHeight: true; clip: true; spacing: 7
                model: root.controller.results; currentIndex: 0
                onModelChanged: currentIndex = 0
                onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    objectName: "launcherResult" + index
                    width: list.width; height: 66; radius: 16
                    color: list.currentIndex === index ? "#70304b68" : "#30132237"
                    border.width: 1; border.color: list.currentIndex === index ? Theme.cyan : Theme.rim
                    RowLayout {
                        anchors.fill: parent; anchors.margins: 12; spacing: 14
                        Item {
                            Layout.preferredWidth: 30; Layout.preferredHeight: 30
                            Image {
                                id: appIcon; anchors.fill: parent
                                source: modelData.icon && !String(modelData.icon).includes(":") ? Quickshell.iconPath(modelData.icon, true) : ""
                                sourceSize.width: 30; sourceSize.height: 30; fillMode: Image.PreserveAspectFit; asynchronous: true
                            }
                            Icon { anchors.centerIn: parent; visible: appIcon.status !== Image.Ready; name: root.controller.mode === "windows" ? "grid" : "package-x-generic"; size: 26; color: Theme.cyan }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 4
                            Text { Layout.fillWidth: true; text: root.controller.label(modelData.title); textFormat: Text.PlainText; elide: Text.ElideRight; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 15; font.weight: Font.DemiBold }
                            Text { Layout.fillWidth: true; text: root.controller.label(modelData.subtitle); textFormat: Text.PlainText; elide: Text.ElideRight; color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11 }
                        }
                        Text { text: root.controller.mode === "windows" ? "Switch ↗" : "Launch ↗"; color: Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11 }
                    }
                    MouseArea { anchors.fill: parent; enabled: !root.controller.busy; cursorShape: Qt.PointingHandCursor; onClicked: root.controller.activate(modelData) }
                    Accessible.role: Accessible.Button; Accessible.name: root.controller.label(modelData.title)
                }
                Text { anchors.centerIn: parent; visible: list.count === 0; text: "Nothing found. Try another search."; color: Theme.inkDim; font.family: Theme.fontSans }
            }
            Text { Layout.fillWidth: true; text: root.controller.error || (root.controller.busy ? "Opening…" : "↑ ↓ to choose   ·   Enter to open   ·   Esc to dismiss"); color: root.controller.error ? Theme.amber : Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11; wrapMode: Text.Wrap; textFormat: Text.PlainText }
        }
    }
}
