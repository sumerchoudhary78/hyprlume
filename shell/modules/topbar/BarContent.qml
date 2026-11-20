import QtQuick
import QtQuick.Controls
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var backend
    required property var clock

    readonly property int side: Theme.px(width >= Theme.px(720) ? 24 : 12)
    readonly property int iconSize: Theme.px(18)
    readonly property int target: Theme.px(36)
    readonly property int band: Theme.barHeight
    readonly property bool compact: width < Theme.px(840)
    readonly property bool stacked: width < nav.implicitWidth + systemControls.naturalWidth + side * 2
        + time.implicitWidth + Theme.px(24)

    implicitHeight: band + (stacked ? systemControls.implicitHeight + Theme.px(9) : 0)
    color: Theme.bar

    function togglePanel(panel) {
        backend.send({op: "panel", panel: panel, open: backend.snapshot.desktop_panel !== panel});
    }

    Rectangle {
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 1
        color: Theme.divider
    }

    Row {
        id: nav
        anchors.left: parent.left
        anchors.leftMargin: root.side
        y: (root.band - height) / 2
        spacing: Theme.px(7)

        Brand {
            anchors.verticalCenter: parent.verticalCenter
            compact: root.width < Theme.px(540)
        }

        Item {
            width: Theme.px(9)
            height: 1
            visible: !root.compact
        }

        GlassButton {
            objectName: "openLauncher"
            iconName: "system-search"
            text: root.compact ? "" : "Apps"
            quiet: true
            height: root.target
            Accessible.name: "Open application and window search"
            ToolTip.visible: hovered && root.compact
            ToolTip.text: Accessible.name
            ToolTip.delay: 550
            onClicked: root.backend.send({op: "panel", panel: "launcher", open: true})
        }

        GlassButton {
            objectName: "quickCommand"
            iconName: "utilities-terminal"
            text: root.compact ? "" : "Command"
            height: root.target
            Accessible.name: "Open command panel"
            ToolTip.visible: hovered && root.compact
            ToolTip.text: Accessible.name
            ToolTip.delay: 550
            onClicked: root.backend.send({op: "toggle"})
        }

        GlassButton {
            objectName: "stopRecording"
            visible: root.backend.capture.recording
            iconName: "stop"
            text: "Stop recording"
            compact: true
            height: root.target
            primary: true
            accent: Theme.rose
            onClicked: root.backend.send({op: "capture", request: {action: "stop"}})
        }
    }

    Row {
        id: dateAndTime
        readonly property real start: nav.x + nav.width + Theme.px(12)
        readonly property real end: (root.stacked ? root.width - root.side : systemControls.x) - Theme.px(12)
        x: Math.max(start, Math.min((parent.width - width) / 2, end - width))
        y: (root.band - height) / 2
        visible: end - start >= time.implicitWidth
        spacing: Theme.px(10)

        Text {
            id: date
            visible: dateAndTime.end - dateAndTime.start >= date.implicitWidth + time.implicitWidth + dateAndTime.spacing
            text: root.clock.date
            color: Theme.inkDim
            font.family: Theme.fontSans
            font.pixelSize: Theme.fontCaption
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            id: time
            text: root.clock.time
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: Theme.px(14)
            font.weight: Font.DemiBold
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    Flow {
        id: systemControls
        readonly property int naturalWidth: children.length * root.target + Math.max(0, children.length - 1) * spacing
        anchors.right: parent.right
        anchors.rightMargin: root.side
        y: root.stacked ? root.band : (root.band - height) / 2
        width: Math.max(0, Math.min(naturalWidth, root.width - root.side * 2))
        spacing: Theme.px(4)

        IconTile {
            objectName: "openConnectivity"
            iconName: "wifi"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: "Open Wi-Fi and Bluetooth"
            active: root.backend.connectivity.panel_open
            onClicked: root.backend.send({op: "connectivity", request: {action: "toggle"}})
        }

        IconTile {
            objectName: "openAudio"
            iconName: "audio-volume-high"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: "Open sound and media"
            active: root.backend.snapshot.desktop_panel === "audio"
            onClicked: root.togglePanel("audio")
        }

        IconTile {
            objectName: "openNotifications"
            iconName: "bell"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: root.backend.notifications.unread > 0
                ? "Notifications · " + root.backend.notifications.unread + " unread" : "Open notifications"
            active: root.backend.notifications.panel_open || root.backend.notifications.dnd
            onClicked: root.backend.send({op: "notifications", request: {action: "toggle"}})
            StatusDot {
                anchors { right: parent.right; top: parent.top; margins: Theme.px(4) }
                visible: root.backend.notifications.unread > 0
                size: Theme.px(5)
                color: Theme.amber
                halo: false
            }
        }

        IconTile {
            objectName: "toggleVoice"
            iconName: active ? "audio-input-microphone" : "microphone-off"
            size: root.target; iconSize: root.iconSize
            quiet: true
            active: root.backend.snapshot.voice !== "off"
            tooltip: active ? "Turn microphone off" : "Enable wake word"
            onClicked: root.backend.send({op: "voice", mode: active ? "off" : "wake"})
        }

        IconTile {
            objectName: "openTray"
            iconName: "grid"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: "Open application tray"
            active: root.backend.snapshot.desktop_panel === "tray"
            onClicked: root.togglePanel("tray")
        }

        IconTile {
            objectName: "openDesktopTools"
            iconName: "layers"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: "Open clipboard and screen capture"
            active: ["clipboard", "capture", "storage"].includes(root.backend.snapshot.desktop_panel)
            onClicked: root.backend.send({op: "panel", panel: "clipboard", open: true})
        }

        IconTile {
            objectName: "openSettings"
            iconName: "preferences-system"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: "Open settings"
            active: root.backend.snapshot.desktop_panel === "settings"
            onClicked: root.backend.send({op: "panel", panel: "settings", open: true})
        }

        IconTile {
            objectName: "lockSession"
            iconName: "lock"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: enabled ? "Lock session" : root.backend.locking.message || "Lock service unavailable"
            active: root.backend.locking.active
            enabled: root.backend.locking.available
            onClicked: root.backend.send({op: "lock"})
        }

        IconTile {
            objectName: "openPower"
            iconName: "power"
            size: root.target; iconSize: root.iconSize
            quiet: true
            tooltip: "Open power and session controls"
            active: root.backend.snapshot.desktop_panel === "power"
            onClicked: root.togglePanel("power")
        }
    }
}
