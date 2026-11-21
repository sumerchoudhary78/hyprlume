import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var backend
    required property var shellState
    spacing: Theme.px(14)

    function openPanel(name) {
        backend.send({op: "panel", panel: name, open: true});
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.px(6)
        IconTile {
            objectName: "quickMute"
            quiet: true; size: Theme.px(36)
            iconName: root.shellState.muted ? "volume-muted" : "audio-volume-high"
            tooltip: root.shellState.muted ? "Unmute output" : "Mute output"
            enabled: root.shellState.hasSound
            onClicked: root.backend.audio.toggleMute(root.shellState.sink)
        }
        GlassSlider {
            objectName: "quickVolume"
            Layout.fillWidth: true
            enabled: root.shellState.hasSound
            value: enabled ? root.shellState.sink.audio.volume : 0
            Accessible.name: "Output volume"
            onMoved: root.backend.audio.setVolume(root.shellState.sink, value)
        }
        Text {
            Layout.preferredWidth: Theme.px(42)
            horizontalAlignment: Text.AlignRight
            text: root.shellState.hasSound ? Math.round(root.shellState.sink.audio.volume * 100) + "%" : "—"
            color: Theme.inkDim; font.family: Theme.fontMono; font.pixelSize: Theme.fontCaption
        }
    }
    GridLayout {
        Layout.fillWidth: true
        columns: width >= Theme.px(300) ? 2 : 1
        rowSpacing: Theme.px(7); columnSpacing: Theme.px(7)
        enabled: root.backend.online

        QuickTile {
            objectName: "openConnectivity"
            iconName: "wifi"; text: "Connections"; detail: root.shellState.wifiLabel
            onClicked: root.backend.send({op: "connectivity", request: {action: "toggle"}})
        }
        QuickTile {
            objectName: "openAudio"
            iconName: "audio-volume-high"; text: "Sound"
            detail: root.shellState.hasSound ? root.backend.audio.label(root.shellState.sink) : "Sound & media"
            onClicked: root.openPanel("audio")
        }
        QuickTile {
            objectName: "toggleVoice"
            iconName: highlighted ? "audio-input-microphone" : "microphone-off"
            text: "Microphone"; highlighted: root.backend.snapshot.voice !== "off"
            Accessible.checkable: true; Accessible.checked: highlighted
            detail: !highlighted ? "Off" : root.backend.snapshot.voice === "dictate" ? "Dictating" : "Wake word on"
            onClicked: root.backend.send({op: "voice", mode: highlighted ? "off" : "wake"})
        }
        QuickTile {
            objectName: "quickDnd"
            iconName: "moon"; text: "Do not disturb"; highlighted: root.backend.notifications.dnd
            Accessible.checkable: true; Accessible.checked: highlighted
            detail: highlighted ? "On" : "Off"
            enabled: root.backend.notifications.available
            onClicked: root.backend.send({op: "notifications", request: {action: "dnd", enabled: !highlighted}})
        }
        QuickTile {
            objectName: "openDesktopTools"
            iconName: "layers"; text: "Desktop tools"; detail: "Clipboard & capture"
            onClicked: root.openPanel("clipboard")
        }
        QuickTile {
            objectName: "openTray"
            iconName: "grid"; text: "App tray"; detail: "Background apps"
            onClicked: root.openPanel("tray")
        }
    }
    Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider }
    RowLayout {
        Layout.fillWidth: true
        enabled: root.backend.online
        GlassButton {
            objectName: "openSettings"
            compact: true; quiet: true; iconName: "preferences-system"; text: "Settings"
            onClicked: root.openPanel("settings")
        }
        Item { Layout.fillWidth: true }
        IconTile {
            objectName: "lockSession"
            quiet: true; size: Theme.px(36); iconName: "lock"
            tooltip: enabled ? "Lock session" : root.backend.locking.message
            enabled: root.backend.locking.available
            onClicked: root.backend.send({op: "lock"})
        }
        IconTile {
            objectName: "openPower"
            quiet: true; size: Theme.px(36); iconName: "power"; tooltip: "Power and session"
            onClicked: root.openPanel("power")
        }
    }
    Text {
        Layout.fillWidth: true
        visible: !root.backend.online || !!root.backend.error || !!root.backend.audio?.error
        text: !root.backend.online ? "Desktop service offline" : root.backend.error || root.backend.audio?.error || ""
        textFormat: Text.PlainText; wrapMode: Text.Wrap
        color: Theme.rose; font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption
    }
}
