import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: panel
    property var snapshot: ({phase: "ready", message: "Your desktop, within reach.", transcript: "", actions: [], receipts: [], confirmation: null, panel_open: true, voice: "off", workspace: null, active_app: ""})
    property string connectionError: ""
    signal request(var operation)
    signal dismissed()
    implicitWidth: 620
    implicitHeight: content.implicitHeight + 48
    radius: 24
    border.width: 1
    border.color: "#60758699"
    gradient: Gradient {
        GradientStop { position: 0; color: "#f0253447" }
        GradientStop { position: 0.55; color: "#f01b2839" }
        GradientStop { position: 1; color: "#f015202e" }
    }
    readonly property bool busy: ["planning", "executing", "speaking"].indexOf(snapshot.phase) >= 0
    readonly property bool review: snapshot.confirmation !== null && snapshot.confirmation !== undefined

    function submit(execute) {
        if (command.text.trim().length && !busy && !review) panel.request({op: "command", text: command.text.trim(), dry_run: !execute})
    }

    ColumnLayout {
        id: content
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 24 }
        spacing: 20
        RowLayout {
            spacing: 12
            Rectangle {
                width: 38; height: 38; radius: 12
                color: "#244740"
                border.color: "#537d72"
                Text { anchors.centerIn: parent; text: "H"; font.pixelSize: 20; font.weight: Font.Medium; color: "#a5e3d2" }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Text { text: "Hyprlume"; color: "#f2f6fa"; font.pixelSize: 21; font.weight: Font.Medium }
                Text { text: "A LITTLE LIGHT FOR YOUR DESKTOP"; color: "#a3b5c9"; font.pixelSize: 10; font.letterSpacing: 1.1; Layout.fillWidth: true; elide: Text.ElideRight }
            }
            Item { Layout.fillWidth: true }
            LumeButton { text: "×"; compact: true; Accessible.name: "Close command panel"; onClicked: panel.dismissed() }
        }
        RowLayout {
            spacing: 8
            Rectangle { width: 7; height: 7; radius: 4; color: panel.snapshot.phase === "error" ? "#f1b5aa" : "#a5e3d2" }
            Text { text: panel.snapshot.phase.toUpperCase(); color: "#b0c0d2"; font.pixelSize: 11; font.letterSpacing: 1.3 }
            Item { Layout.fillWidth: true }
            Text { text: panel.snapshot.workspace !== null ? "WORKSPACE " + panel.snapshot.workspace : "HYPRPILOT"; color: "#b0c0d2"; font.pixelSize: 11; font.letterSpacing: 1 }
        }
        Text {
            text: panel.snapshot.message
            color: "#f0f4f8"
            font.pixelSize: 19
            font.weight: Font.Medium
            wrapMode: Text.Wrap
            Layout.fillWidth: true
            Accessible.role: Accessible.StaticText
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 10
            TextField {
                id: command
                objectName: "commandInput"
                Layout.fillWidth: true
                implicitHeight: 48
                placeholderText: "workspace 2 · focus firefox · volume 30"
                placeholderTextColor: "#96a8bc"
                color: "#eef5fa"
                font.pixelSize: 14
                selectByMouse: true
                maximumLength: 512
                enabled: !panel.busy && !panel.review
                leftPadding: 14; rightPadding: 14
                background: Rectangle { radius: 12; color: "#15202f"; border.width: 1; border.color: command.activeFocus ? "#a5e3d2" : "#4e6175" }
                onAccepted: panel.submit(false)
            }
            RowLayout {
                spacing: 8
                LumeButton { objectName: "previewButton"; text: "Preview"; primary: true; enabled: command.text.trim().length > 0 && !panel.busy && !panel.review; onClicked: panel.submit(false) }
                LumeButton { objectName: "runButton"; text: "Run direct command"; enabled: command.text.trim().length > 0 && !panel.busy && !panel.review; onClicked: panel.submit(true) }
                Item { Layout.fillWidth: true }
            }
        }
        ColumnLayout {
            visible: panel.snapshot.transcript.length > 0
            Layout.fillWidth: true
            spacing: 6
            Text { text: panel.snapshot.voice === "dictate" ? "DICTATION · SELECT TO COPY" : "YOUR REQUEST"; font.pixelSize: 10; font.letterSpacing: 1.1; color: "#a3b5c9" }
            TextArea {
                objectName: "transcript"
                text: panel.snapshot.transcript
                Layout.fillWidth: true
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.Wrap
                color: "#e2edf5"
                font.pixelSize: 14
                padding: 0
                background: null
            }
        }
        ColumnLayout {
            visible: panel.snapshot.actions.length > 0
            Layout.fillWidth: true
            spacing: 10
            Repeater {
                model: panel.snapshot.actions
                delegate: RowLayout {
                    required property string modelData
                    required property int index
                    spacing: 12
                    Text { text: (index + 1).toString().padStart(2, "0"); color: "#a5e3d2"; font.pixelSize: 12 }
                    Text { text: modelData; Layout.fillWidth: true; color: "#eef4f8"; font.pixelSize: 14; wrapMode: Text.Wrap }
                }
            }
            RowLayout {
                visible: panel.review
                spacing: 8
                LumeButton { objectName: "confirmButton"; text: "Confirm action"; primary: true; enabled: !panel.busy; onClicked: panel.request({op: "confirm", token: panel.snapshot.confirmation}) }
                LumeButton { text: "Dismiss"; enabled: !panel.busy; onClicked: panel.request({op: "cancel"}) }
                Item { Layout.fillWidth: true }
            }
        }
        ColumnLayout {
            visible: panel.snapshot.receipts.length > 0
            Layout.fillWidth: true
            spacing: 6
            Repeater {
                model: panel.snapshot.receipts
                delegate: Text { required property string modelData; text: "✓  " + modelData; color: "#b9e7d8"; font.pixelSize: 13; wrapMode: Text.Wrap; Layout.fillWidth: true }
            }
        }
        Text { visible: panel.connectionError.length > 0; text: panel.connectionError; color: "#f1b5aa"; font.pixelSize: 13; Layout.fillWidth: true; wrapMode: Text.Wrap }
        Rectangle { Layout.fillWidth: true; height: 1; color: "#3c4d60" }
        Flow {
            Layout.fillWidth: true
            spacing: 8
            LumeButton { text: panel.snapshot.voice === "wake" ? "Wake word on" : "Enable wake word"; compact: true; primary: panel.snapshot.voice === "wake"; onClicked: panel.request({op: "voice", mode: panel.snapshot.voice === "wake" ? "off" : "wake"}) }
            LumeButton { text: "Dictation"; compact: true; primary: panel.snapshot.voice === "dictate"; onClicked: panel.request({op: "voice", mode: "dictate"}) }
            LumeButton { text: "Mic off"; compact: true; primary: panel.snapshot.voice === "off"; onClicked: panel.request({op: "voice", mode: "off"}) }
        }
        Text { text: "LOCAL VOICE  /  ENGLISH  /  " + (panel.snapshot.voice === "off" ? "MICROPHONE OFF" : "MICROPHONE ACTIVE"); color: "#a3b5c9"; font.pixelSize: 10; font.letterSpacing: 1; Layout.fillWidth: true; wrapMode: Text.Wrap }
    }
    Keys.onEscapePressed: panel.dismissed()
    function focusInput() { command.forceActiveFocus() }
}
