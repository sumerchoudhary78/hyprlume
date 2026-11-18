import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var backend

    readonly property var state: backend.snapshot.displays || {}
    readonly property bool busy: !!state.busy
    readonly property int revision: state.revision || 0
    readonly property var outputs: state.outputs || []
    readonly property var selection: outputs.find(output => output.name === selectedName)
    readonly property var current: draft[selectedName]
    property string selectedName: ""
    property var draft: ({})
    property bool edited: false
    property int pending: 0
    property string error: ""
    property int secondsLeft: 0
    spacing: 13

    function reset() {
        const next = {};
        for (const output of outputs) {
            if (output.active) next[output.name] = Object.assign({}, output.active);
        }
        draft = next;
        if (!selection) selectedName = outputs.find(output => output.active)?.name || outputs[0]?.name || "";
        edited = false;
    }

    function edit(key, value) {
        draft = Object.assign({}, draft, {[selectedName]: Object.assign({}, current, {[key]: value})});
        edited = true;
    }

    function send(request) {
        error = "";
        pending = backend.send({op: "display", request: request}) || 0;
        if (!pending) error = backend.error;
    }

    function updateCountdown() {
        secondsLeft = Math.max(0, Math.ceil(((state.preview?.expires_at || 0) - Date.now()) / 1000));
    }

    onRevisionChanged: reset()
    onStateChanged: updateCountdown()
    onBusyChanged: if (!busy && !state.preview) reset()
    Component.onCompleted: {
        reset();
        send({action: "refresh"});
    }
    Component.onDestruction: {
        if (state.preview) backend.send({op: "display", request: {action: "revert", token: state.preview.token}});
    }

    Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (id !== root.pending) return;
            root.pending = 0;
            if (!ok) root.error = error;
        }
    }

    Timer {
        running: !!root.state.preview
        repeat: true
        interval: 200
        onTriggered: root.updateCountdown()
    }

    ColumnLayout {
        visible: !!root.state.preview
        Layout.fillWidth: true

        Text {
            text: "Keep this layout? Reverting in " + root.secondsLeft + " s"
            color: Theme.amber
            font.family: Theme.fontSans
            font.pixelSize: 16
        }
        RowLayout {
            GlassButton {
                objectName: "displayRevert"
                text: "Revert"
                enabled: !root.pending
                onClicked: root.send({action: "revert", token: root.state.preview.token})
            }
            GlassButton {
                objectName: "displayKeep"
                text: "Keep layout"
                primary: true
                enabled: !root.pending && root.secondsLeft > 0
                onClicked: root.send({action: "keep", token: root.state.preview.token})
            }
        }
    }

    Text {
        Layout.fillWidth: true
        text: root.error || root.state.error || root.state.message || "Reading connected displays…"
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
        color: root.error || root.state.error ? Theme.amber : Theme.inkDim
        font.pixelSize: 12
    }

    DisplayMap {
        Layout.fillWidth: true
        implicitHeight: 154
        settings: Object.values(root.draft)
        selected: root.selectedName
        onSelectedOutput: name => root.selectedName = name
    }

    Flow {
        Layout.fillWidth: true
        spacing: 7
        Repeater {
            model: root.outputs
            GlassButton {
                required property var modelData
                text: modelData.name
                primary: modelData.name === root.selectedName
                compact: true
                onClicked: root.selectedName = modelData.name
            }
        }
    }

    Text {
        Layout.fillWidth: true
        visible: !!root.selection
        text: root.selection?.label || root.selectedName
        textFormat: Text.PlainText
        elide: Text.ElideRight
        color: Theme.ink
        font.pixelSize: 14
    }

    GridLayout {
        visible: !!root.current
        enabled: !root.state.busy && !root.pending
        Layout.fillWidth: true
        columns: width > 440 ? 2 : 1
        columnSpacing: 12
        rowSpacing: 10

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { text: "Resolution and refresh"; color: Theme.inkDim; font.pixelSize: 12 }
            ComboBox {
                objectName: "displayMode"
                Layout.fillWidth: true
                Layout.preferredHeight: 39
                model: root.selection?.modes || []
                currentIndex: model.indexOf(root.current?.mode || "")
                onActivated: root.edit("mode", currentText)
            }
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "displayScale"
            label: "Scale (0.5–4)"
            value: root.current ? String(root.current.scale) : ""
            limit: 6
            onEdited: value => root.edit("scale", Number(value))
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "displayX"
            label: "Horizontal position"
            value: root.current ? String(root.current.x) : ""
            limit: 6
            onEdited: value => root.edit("x", Number(value))
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "displayY"
            label: "Vertical position"
            value: root.current ? String(root.current.y) : ""
            limit: 6
            onEdited: value => root.edit("y", Number(value))
        }
    }

    Text {
        visible: !!root.selection && !root.current
        text: "This output is disabled in Hyprland."
        color: Theme.inkDim
        font.pixelSize: 12
    }

    Flow {
        Layout.fillWidth: true
        spacing: 8
        GlassButton {
            objectName: "displayApply"
            text: "Preview layout"
            primary: true
            enabled: root.edited && !root.state.busy && !root.pending
            onClicked: root.send({action: "preview", revision: root.revision, changes: Object.values(root.draft)})
        }
        GlassButton {
            text: "Discard edits"
            enabled: root.edited && !root.state.busy
            onClicked: root.reset()
        }
        GlassButton {
            text: "Refresh"
            enabled: !root.state.busy && !root.pending
            onClicked: root.send({action: "refresh"})
        }
    }
}
