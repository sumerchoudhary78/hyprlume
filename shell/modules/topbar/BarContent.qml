import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import qs.components
import qs.theme

GlassSurface {
    id: root
    required property var backend
    required property var state
    property bool presentation: false
    reducedEffects: root.state.reducedEffects
    radius: 23
    glowColor: "#496983"
    secondaryColor: "#354154"
    glowing: false
    tintTop: "#ee111827"
    tint: "#eb09101d"
    readonly property bool compact: width < 1250 || backend.capture.recording
    readonly property var modes: [
        {label:"Terminal",icon:"utilities-terminal"}, {label:"Actions",icon:"workflow"},
        {label:"Agent",icon:"sparkles"}, {label:"Commands",icon:"text-x-script"},
        {label:"Containers",icon:"package-x-generic"}, {label:"Network",icon:"cloud"}
    ]
    RowLayout {
        id: controls
        readonly property real availableWidth: Math.max(1, parent.width - 35)
        anchors.left: parent.left
        anchors.leftMargin: 20
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(availableWidth, implicitWidth)
        scale: availableWidth / width
        transformOrigin: Item.Left
        spacing: 8
        Brand { presentation: root.presentation; compact: root.compact }
        Row {
            spacing: 2
            Repeater {
                model: root.modes
                ModeTab {
                    required property var modelData
                    required property int index
                    objectName: "mode" + index
                    label: modelData.label; iconName: modelData.icon
                    active: root.state.mode === index
                    showLabel: !root.compact
                    onClicked: {
                        root.state.mode = index;
                        if (index === 3) root.backend.send({op:"toggle"});
                        if (index === 5) root.backend.send({op:"connectivity",request:{action:"toggle"}});
                    }
                }
            }
        }
        Item { Layout.fillWidth: true }
        SearchBox {
            onActivated: root.backend.send({op:"panel",panel:"launcher",open:true})
        }
        Item {
            implicitWidth: 34; implicitHeight: 34
            IconTile {
                objectName: "openNotifications"
                anchors.fill: parent; iconName: "bell"; size: 34; iconSize: 19
                active: root.backend.notifications.panel_open || root.backend.notifications.dnd
                accent: root.backend.notifications.dnd ? Theme.violet : Theme.cyan
                Accessible.name: "Open notification centre"
                onClicked: root.backend.send({op:"notifications",request:{action:"toggle"}})
            }
            StatusDot { anchors.right: parent.right; anchors.top: parent.top; visible: root.backend.notifications.unread > 0; size: 5; color: Theme.amber }
        }
        IconTile {
            objectName: "openAudio"
            iconName: "audio-volume-high"; size: 34; iconSize: 20
            active: root.backend.snapshot.desktop_panel === "audio"
            Accessible.name: "Open sound and media"
            onClicked: root.backend.send({op:"panel",panel:"audio",open:root.backend.snapshot.desktop_panel !== "audio"})
        }
        GlassButton {
            objectName: "stopRecording"; visible: root.backend.capture.recording
            text: "Stop recording"; compact: true; primary: true
            onClicked: root.backend.send({op:"capture",request:{action:"stop"}})
        }
        IconTile {
            objectName:"openDesktopTools"; iconName:"layers"; size:34; iconSize:19
            Accessible.name:"Open clipboard and screen capture"
            onClicked:root.backend.send({op:"panel",panel:"clipboard",open:true})
        }
        IconTile {
            objectName:"openSettings"; iconName:"preferences-system"; size:34; iconSize:19
            Accessible.name:"Open settings"
            onClicked:root.backend.send({op:"panel",panel:"settings",open:true})
        }
        IconTile {
            objectName: "openTray"
            iconName: "grid"; size: 34; iconSize: 19
            active: root.backend.snapshot.desktop_panel === "tray"
            Accessible.name: "Open application tray"
            onClicked: root.backend.send({op:"panel",panel:"tray",open:root.backend.snapshot.desktop_panel !== "tray"})
        }
        IconTile {
            objectName:"lockSession"; iconName:"lock"; size:34; iconSize:19
            active:root.backend.locking.active; enabled:root.backend.locking.available
            Accessible.name:"Lock session"
            onClicked:root.backend.send({op:"lock"})
            HoverHandler {id:lockHover}
            ToolTip.visible:lockHover.hovered
            ToolTip.text:root.backend.locking.message
        }
        IconTile {
            objectName: "openPower"
            iconName: "power"; size: 34; iconSize: 19
            active: root.backend.snapshot.desktop_panel === "power"
            Accessible.name: "Open power and session controls"
            onClicked: root.backend.send({op:"panel",panel:"power",open:root.backend.snapshot.desktop_panel !== "power"})
        }
        IconTile {
            iconName: "audio-input-microphone"; size: 34; iconSize: 20
            active: root.backend.snapshot.voice !== "off"; accent: Theme.violet
            onClicked: root.backend.send({op:"voice",mode:root.backend.snapshot.voice === "off" ? "wake" : "off"})
            Accessible.name: "Toggle microphone"
        }
        Rectangle {
            objectName: "openConnectivity"
            implicitWidth: 35; implicitHeight: 35; radius: 18; color: "#071633"; border.width: 1; border.color: Theme.violet
            Icon { anchors.centerIn: parent; name: "globe"; color: "#9cd7ff"; size: 26 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.backend.send({op:"connectivity",request:{action:"toggle"}}) }
            Accessible.role: Accessible.Button
            Accessible.name: "Open Wi-Fi and Bluetooth"
        }
    }
}
