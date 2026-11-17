import QtQuick
import Quickshell

QtObject {
    id: root
    property string name: "Sample device"
    property string nickname: name
    property string description: name
    property bool isSink: true
    property bool isStream: false
    property bool ready: true
    property QtObject audio: QtObject {
        property real volume: 0.64
        property bool muted: false
        onVolumeChanged: root.report("volume", volume)
        onMutedChanged: root.report("mute", muted)
    }
    function report(action, value) {
        if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"audio",action:action,node:root.name,value:value}));
    }
}
