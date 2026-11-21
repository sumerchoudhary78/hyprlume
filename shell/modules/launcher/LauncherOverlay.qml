import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Templates as T
import Quickshell
import qs.components
import qs.theme

FocusScope {
    id: root
    required property var backend
    property bool reducedEffects: false
    readonly property var controller: backend.launcher
    readonly property bool windows: backend.snapshot.desktop_panel === "switcher"
    focus: true
    Keys.onEscapePressed: close()

    function close() {
        const panel = backend.snapshot.desktop_panel;
        if (panel === "launcher" || panel === "switcher")
            backend.send({op: "panel", panel: panel, open: false});
    }
    function changeMode(panel) {
        backend.send({op: "panel", panel: panel, open: true});
        search.forceActiveFocus();
    }

    Connections { target: root.controller; function onActivated() { root.close(); } }
    Binding { target: root.controller; property: "mode"; value: root.windows ? "windows" : "apps" }

    Rectangle {
        anchors.fill: parent
        color: "#660e0d13"
        MouseArea { anchors.fill: parent; onClicked: root.close() }
    }
    GlassSurface {
        id: palette
        anchors.horizontalCenter: parent.horizontalCenter
        y: Math.round((root.height - height) * 0.36)
        width: Math.min(Theme.px(584), root.width - Theme.px(24))
        height: Math.min(root.height - Theme.px(24), layout.implicitHeight + Theme.px(32))
        transform: PanelEntrance { reducedEffects: root.reducedEffects }
        radius: Theme.radiusCard
        Accessible.role: Accessible.Dialog
        Accessible.name: root.windows ? "Switch windows" : "Launch an application"

        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
        ColumnLayout {
            id: layout
            anchors.fill: parent; anchors.margins: Theme.px(16)
            spacing: Theme.px(9)

            RowLayout {
                Layout.fillWidth: true; spacing: Theme.px(10)
                Icon { name: "system-search"; size: Theme.px(21); color: Theme.cyan }
                TextField {
                    id: search
                    objectName: "launcherSearch"
                    Layout.fillWidth: true; Layout.minimumWidth: 0
                    Layout.preferredHeight: Theme.px(42)
                    padding: 0; background: null
                    placeholderText: root.windows ? "Find an open window…" : "Search applications…"
                    Accessible.name: placeholderText
                    text: root.controller.query
                    maximumLength: 128; selectByMouse: true
                    font.family: Theme.fontSans; font.pixelSize: Theme.fontTitle
                    color: Theme.ink; placeholderTextColor: Theme.inkDim
                    selectionColor: Theme.cyan; selectedTextColor: Theme.onAccent
                    onTextEdited: root.controller.query = text
                    Keys.onDownPressed: list.currentIndex = Math.min(list.count - 1, list.currentIndex + 1)
                    Keys.onUpPressed: list.currentIndex = Math.max(0, list.currentIndex - 1)
                    onAccepted: root.controller.activate(root.controller.results[list.currentIndex])
                    Component.onCompleted: forceActiveFocus()
                }
                IconTile {
                    objectName: "closeLauncher"
                    quiet: true; size: Theme.px(32); iconName: "close"; tooltip: "Close launcher"
                    onClicked: root.close()
                }
            }
            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider }
            RowLayout {
                Layout.fillWidth: true; spacing: Theme.px(4)
                GlassButton {
                    objectName: "launcherApps"
                    text: "Applications"; compact: true; primary: !root.windows; quiet: true
                    onClicked: root.changeMode("launcher")
                }
                GlassButton {
                    objectName: "launcherWindows"
                    text: "Windows"; compact: true; primary: root.windows; quiet: true
                    onClicked: root.changeMode("switcher")
                }
            }
            ListView {
                id: list
                // Keep the search field stationary while the result count changes.
                Layout.fillWidth: true; Layout.fillHeight: true; Layout.preferredHeight: Theme.px(280)
                clip: true; spacing: Theme.px(2)
                model: root.controller.results
                currentIndex: 0
                onModelChanged: currentIndex = root.controller.results.length ? 0 : -1
                onCurrentIndexChanged: if (currentIndex >= 0) positionViewAtIndex(currentIndex, ListView.Contain)
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

                delegate: T.Button {
                    id: result
                    required property var modelData
                    required property int index
                    objectName: "launcherResult" + index
                    width: list.width - (list.contentHeight > list.height ? Theme.px(10) : 0)
                    height: Theme.px(53)
                    padding: Theme.px(9)
                    enabled: !root.controller.busy
                    hoverEnabled: true
                    focusPolicy: Qt.NoFocus
                    Accessible.name: root.controller.label(modelData.title)
                    onClicked: root.controller.activate(modelData)

                    background: Rectangle {
                        radius: Theme.px(8)
                        color: result.down ? Theme.pressed : result.hovered ? Theme.hover
                            : result.index === list.currentIndex ? Theme.selection : "transparent"
                        border.width: result.index === list.currentIndex ? 1 : 0
                        border.color: Theme.rimStrong
                    }
                    contentItem: RowLayout {
                        spacing: Theme.px(11)
                        Item {
                            Layout.preferredWidth: Theme.px(26); Layout.preferredHeight: Theme.px(26)
                            Image {
                                id: appIcon
                                anchors.fill: parent; fillMode: Image.PreserveAspectFit; asynchronous: true
                                source: result.modelData.icon && !String(result.modelData.icon).includes(":")
                                    ? Quickshell.iconPath(result.modelData.icon, true) : ""
                                sourceSize: Qt.size(width * Screen.devicePixelRatio, height * Screen.devicePixelRatio)
                            }
                            Icon {
                                anchors.centerIn: parent
                                visible: appIcon.status !== Image.Ready
                                name: root.windows ? "workspace" : "grid"; size: Theme.px(20); color: Theme.inkDim
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; Layout.minimumWidth: 0; spacing: 0
                            Text {
                                Layout.fillWidth: true
                                text: root.controller.label(result.modelData.title); textFormat: Text.PlainText; elide: Text.ElideRight
                                font.family: Theme.fontSans; font.pixelSize: Theme.fontBody; font.weight: Font.DemiBold; color: Theme.ink
                            }
                            Text {
                                Layout.fillWidth: true; visible: !!text
                                text: root.controller.label(result.modelData.subtitle); textFormat: Text.PlainText; elide: Text.ElideRight
                                font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption; color: Theme.inkDim
                            }
                        }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                }
                Text {
                    anchors.centerIn: parent
                    visible: list.count === 0
                    text: root.controller.query ? "No results" : root.windows ? "No open windows" : "No applications"
                    color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: Theme.fontBody
                }
            }
            Text {
                Layout.fillWidth: true
                text: root.controller.error || (root.controller.busy ? "Opening…" : "↑ ↓  Select     Enter  Open     Esc  Close")
                textFormat: Text.PlainText; wrapMode: Text.Wrap
                color: root.controller.error ? Theme.rose : Theme.inkFaint
                font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption
            }
        }
    }
}
