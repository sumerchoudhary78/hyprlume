import QtQuick
import QtQuick.Layouts
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
    readonly property var modes: [
        {label:"Terminal",icon:"utilities-terminal"}, {label:"Actions",icon:"workflow"},
        {label:"Agent",icon:"sparkles"}, {label:"Commands",icon:"text-x-script"},
        {label:"Containers",icon:"package-x-generic"}, {label:"Network",icon:"cloud"}
    ]
    RowLayout {
        anchors.fill: parent; anchors.leftMargin: 20; anchors.rightMargin: 15; spacing: 12
        Brand { presentation: root.presentation }
        Item { Layout.preferredWidth: 7 }
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
                    showLabel: root.width > 1010
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
            onActivated: root.backend.send({op:"toggle"})
        }
        Item {
            width: 34; height: 34
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
            iconName: "audio-input-microphone"; size: 34; iconSize: 20
            active: root.backend.snapshot.voice !== "off"; accent: Theme.violet
            onClicked: root.backend.send({op:"voice",mode:root.backend.snapshot.voice === "off" ? "wake" : "off"})
            Accessible.name: "Toggle microphone"
        }
        Rectangle {
            objectName: "openConnectivity"
            width: 35; height: 35; radius: 18; color: "#071633"; border.width: 1; border.color: Theme.violet
            Icon { anchors.centerIn: parent; name: "globe"; color: "#9cd7ff"; size: 26 }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.backend.send({op:"connectivity",request:{action:"toggle"}}) }
            Accessible.role: Accessible.Button
            Accessible.name: "Open Wi-Fi and Bluetooth"
        }
    }
}
