import QtQuick
import QtQuick.Layouts
import qs.components
import qs.models
import qs.theme

GlassSurface {
    id: root
    required property var backend
    property bool focused: false
    onFocusedChanged: { if (focused) agentInput.forceActiveFocus(); }
    CommandState {
        id: commandState
        snapshot: root.backend.snapshot
    }
    glowColor: root.focused ? Theme.violet : Theme.cyan
    secondaryColor: Theme.violet

    function ask(text) {
        if (!commandState.blocked && text.trim().length > 0) backend.send({op:"command",text:text.trim(),dry_run:true});
    }
    PanelHeader {
        x: 17; y: 17; width: parent.width - 130
        title: "Shell Agent"; iconName: "sparkles"; accent: Theme.cyan

    }
    Text { x: 138; y: 20; text: root.backend.online ? "● Online" : "● Offline"; color: root.backend.online ? Theme.mint : Theme.rose; font.family: Theme.fontSans; font.pixelSize: 10 }
    Image {
        x: parent.width - 140; y: -52; width: 153; height: 149
        source: "../../assets/mascot/pilot.png"
        sourceSize: Qt.size(360, 360)
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        transform: Rotation { origin.x: 80; origin.y: 80; angle: mascotHover.hovered ? -4 : 0 }
        HoverHandler { id: mascotHover }
    }
    Rectangle {
        x: 17; y: 55; width: parent.width - 34; height: 35
        radius: 13; color: "#78081221"; border.width: 1; border.color: "#637dc8ee"
        TextInput {
            id: agentInput; objectName: "agentInput"
            anchors.fill: parent; anchors.leftMargin: 12; anchors.rightMargin: 55
            verticalAlignment: TextInput.AlignVCenter
            color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 13
            clip: true; maximumLength: 512; selectByMouse: true
            enabled: !commandState.blocked && root.backend.online
            onAccepted: { root.ask(text.trim()); text = ""; }
            Text { anchors.verticalCenter: parent.verticalCenter; visible: !agentInput.text; text: "How can I help today?"; color: Theme.inkDim; font: agentInput.font }
            Accessible.name: "Ask the local desktop agent"
        }
    }
    Column {
        x: 31; y: 103; width: parent.width - 62; spacing: 6
        ActionRow { width: parent.width; label: "Open my terminal"; iconName: "code"; enabled: !commandState.blocked; onClicked: root.ask("launch terminal") }
        ActionRow { width: parent.width; label: "Switch to workspace two"; iconName: "workflow"; enabled: !commandState.blocked; onClicked: root.ask("workspace 2") }
        ActionRow { width: parent.width; objectName: "agentVolume"; label: "Set volume to 35%"; iconName: "audio-volume-high"; enabled: !commandState.blocked; onClicked: root.ask("volume 35") }
        ActionRow { width: parent.width; label: "Dictate a note"; iconName: "text-x-script"; onClicked: root.backend.send({op:"voice",mode:"dictate"}) }
    }
    Rectangle {
        x: 35; y: parent.height - 48; width: parent.width - 70; height: 36
        radius: 18; color: "#8c051326"; border.width: 1; border.color: "#435894c4"
        WaveBars { x: 39; y: 6; width: parent.width - 55; height: 24; barCount: 44; barWidth: 2; barSpacing: 2; animated: !root.reducedEffects; active: commandState.voice !== "off"; accent: Theme.violet }
        IconTile {
            x: -5; y: -1; size: 38; iconSize: 19
            iconName: "audio-input-microphone"; active: commandState.voice !== "off"; accent: Theme.violet
            onClicked: root.backend.send({op:"voice",mode:commandState.voice === "off" ? "wake" : "off"})
            Accessible.name: commandState.voice === "off" ? "Enable wake word" : "Turn microphone off"
        }
    }
    Text {
        anchors.horizontalCenter: parent.horizontalCenter; anchors.bottom: parent.bottom; anchors.bottomMargin: 4
        text: commandState.voice === "off" ? "MIC OFF · LOCAL VOICE" : commandState.voice === "wake" ? "HEY PILOT · WAKE WORD" : "DICTATION · TEXT ONLY"
        color: Theme.inkFaint; font.family: Theme.fontMono; font.pixelSize: 7; font.letterSpacing: 1
    }
}
