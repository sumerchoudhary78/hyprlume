import QtQuick
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var backend
    required property var clock
    required property var shellState
    property var outputScreen: null
    property string workspaceLabel: ""

    readonly property bool compact: width < Theme.px(760)
    readonly property bool narrow: width < Theme.px(540)
    readonly property int inset: Theme.px(compact ? 6 : 16)
    implicitHeight: Theme.barHeight
    color: Theme.bar

    component PanelButton: BarButton {
        focusPolicy: root.shellState.menu || root.shellState.servicePanelOpen ? Qt.NoFocus : Qt.TabFocus
    }

    Row {
        id: navigation
        anchors.left: parent.left
        anchors.leftMargin: root.inset
        spacing: Theme.px(2)

        PanelButton {
            objectName: "openOverview"
            text: root.narrow ? "" : root.workspaceLabel || "Atlas"
            maximumWidth: status.x - navigation.x - separator.width - launcher.width - command.width
                - navigation.spacing * 3 - clockButton.timeWidth - clockButton.gap * 2
            iconName: "hyprlume"
            iconColor: Theme.atlasCopper
            tooltip: root.workspaceLabel ? "Workspace atlas · " + root.workspaceLabel : "Workspace atlas"
            showChevron: !root.narrow
            enabled: !root.shellState.servicePanelOpen && !root.shellState.locked
            selected: root.shellState.menu === "overview" && root.shellState.panelScreen === root.outputScreen
            onClicked: root.shellState.toggleMenu("overview", root.outputScreen)
        }
        Rectangle {
            id: separator
            anchors.verticalCenter: parent.verticalCenter
            width: 1; height: Theme.px(15)
            color: Theme.divider
        }
        PanelButton {
            id: launcher
            objectName: "openLauncher"
            text: root.compact ? "" : "Apps"
            iconName: "grid"
            tooltip: "Search applications and windows"
            enabled: root.backend.online && !root.shellState.locked
            onClicked: root.shellState.openFromBar(root.outputScreen, {op: "panel", panel: "launcher", open: true})
        }
        PanelButton {
            id: command
            objectName: "quickCommand"
            text: root.compact ? "" : "Command"
            iconName: "utilities-terminal"
            tooltip: "Open command panel"
            enabled: root.backend.online && !root.shellState.locked
            onClicked: root.shellState.openFromBar(root.outputScreen, {op: "toggle"})
        }
    }

    PanelButton {
        id: clockButton
        objectName: "openNotifications"
        readonly property real gap: Theme.px(4)
        readonly property real timeWidth: leftPadding + rightPadding + time.implicitWidth
            + (notificationIcon.visible ? notificationIcon.width + clockRow.spacing : 0)
        readonly property real leftLimit: navigation.x + navigation.width + gap
        readonly property real room: status.x - gap - leftLimit
        x: Math.max(leftLimit, Math.min((root.width - width) / 2, status.x - gap - width))
        tooltip: root.clock.date + " · Notifications" + (root.backend.notifications.dnd ? " · Do not disturb" :
            root.backend.notifications.unread ? " · " + root.backend.notifications.unread + " unread" : "")
        onClicked: root.shellState.openFromBar(root.outputScreen, {op: "notifications", request: {action: "toggle"}})
        contentItem: Row {
            id: clockRow
            spacing: Theme.px(8)
            Text {
                id: date
                anchors.verticalCenter: parent.verticalCenter
                visible: clockButton.room >= implicitWidth + clockRow.spacing + clockButton.timeWidth
                text: root.clock.date
                color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: Theme.px(12)
            }
            Text {
                id: time
                anchors.verticalCenter: parent.verticalCenter
                text: root.clock.time
                color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: Theme.px(14); font.weight: Font.DemiBold
            }
            Icon {
                id: notificationIcon
                anchors.verticalCenter: parent.verticalCenter
                visible: root.backend.online && (root.backend.notifications.unread > 0 || root.backend.notifications.dnd)
                name: root.backend.notifications.dnd ? "moon" : "bell"
                size: Theme.px(13)
                color: Theme.cyan
            }
        }
    }

    Row {
        id: status
        anchors.right: parent.right
        anchors.rightMargin: root.inset
        spacing: Theme.px(3)
        PanelButton {
            objectName: "stopRecording"
            visible: root.backend.online && root.backend.capture.recording
            iconName: "stop"
            text: root.compact ? "" : "Stop recording"
            iconColor: Theme.rose
            tooltip: "Stop screen recording"
            onClicked: root.backend.send({op: "capture", request: {action: "stop"}})
        }
        PanelButton {
            objectName: "activeMicrophone"
            visible: root.backend.online && root.backend.snapshot.voice !== "off"
            iconName: "audio-input-microphone"
            iconColor: Theme.mint
            tooltip: "Turn microphone off"
            onClicked: root.backend.send({op: "voice", mode: "off"})
        }
        PanelButton {
            objectName: "quickSettings"
            tooltip: "Quick settings"
            enabled: !root.shellState.servicePanelOpen && !root.shellState.locked
            selected: root.shellState.menu === "controls" && root.shellState.panelScreen === root.outputScreen
            onClicked: root.shellState.toggleMenu("controls", root.outputScreen)
            contentItem: Row {
                spacing: Theme.px(8)
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.narrow && root.backend.online && root.shellState.wifi.available
                    name: root.shellState.connection ? "wifi" : "wifi-off"
                    size: Theme.px(15); color: Theme.inkDim
                }
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.narrow && root.shellState.hasSound
                    name: root.shellState.muted ? "volume-muted" : "audio-volume-high"
                    size: Theme.px(15); color: Theme.inkDim
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.narrow && root.shellState.hasBattery
                    text: Math.round(root.shellState.battery.percentage) + "%"
                    font.family: Theme.fontSans; font.pixelSize: Theme.px(12); color: Theme.ink
                }
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.narrow && root.shellState.hasBattery
                    name: root.shellState.battery.state === "charging" ? "battery-charging" : "battery"
                    size: Theme.px(17); color: Theme.inkDim
                    Rectangle {
                        visible: root.shellState.hasBattery && root.shellState.battery.state !== "charging"
                        x: parent.size * 0.19; y: parent.size * 0.36
                        width: parent.size * 0.54 * Math.min(100, Math.max(0, root.shellState.battery.percentage)) / 100
                        height: parent.size * 0.28; color: parent.color
                    }
                }
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "sliders"; size: Theme.px(16); color: Theme.ink
                }
            }
        }
    }
    Rectangle {
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 1; color: Theme.divider
    }
}
