import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

Rectangle {
    id: root
    required property var backend
    required property var light
    required property int lightIndex
    property bool pending: false
    implicitHeight: 88; radius: 15; color: "#38152236"; border.width: 1; border.color: Theme.rim
    ColumnLayout {
        anchors.fill: parent; anchors.margins: 12; spacing: 2
        RowLayout {
            Layout.fillWidth: true
            Icon { name: "brightness"; size: 17; color: Theme.amber }
            Text { Layout.fillWidth: true; text: root.light.name; textFormat: Text.PlainText; elide: Text.ElideRight; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 12 }
            Text { text: Math.round(slider.value * 100) + "%"; color: Theme.inkDim; font.family: Theme.fontMono; font.pixelSize: 11 }
        }
        GlassSlider {
            id: slider
            objectName: "brightness" + root.lightIndex
            Layout.fillWidth: true; from: 0.05
            value: root.light.percent / 100
            enabled: root.backend.power.session.available && !root.backend.power.busy && !root.backend.power.prompt
            Accessible.name: "Brightness for " + root.light.name
            onMoved: { root.pending = true; commit.restart(); }
            onPressedChanged: if (!pressed && root.pending) commit.restart()
        }
    }
    Timer {
        id: commit
        interval: 160
        onTriggered: {
            if (!root.pending || slider.pressed || !slider.enabled) return;
            root.pending = false;
            root.backend.send({op:"power",request:{action:"brightness",device:root.light.id,percent:Math.round(slider.value * 100)}});
        }
    }
}
