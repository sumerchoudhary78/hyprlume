import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var backend
    required property var profile
    required property int revision
    required property bool saved
    signal edited(string key, var value)

    property var keyboards: []
    property string message: ""
    property string error: ""
    property string profilePath: ""
    property string action: ""
    property int pending: 0

    function send(name) {
        action = name;
        error = "";
        pending = backend.send({op: "input", request: name === "status" || name === "next_layout"
            ? {action: name} : {action: name, revision: revision}}) || 0;
        if (!pending) error = backend.error;
    }

    Component.onCompleted: send("status")
    Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (id !== root.pending) return;
            root.pending = 0;
            if (!ok) { root.error = error; return; }
            if (root.action === "status") root.keyboards = result.keyboards || [];
            else if (root.action === "next_layout" || root.action === "apply") {
                root.send("status");
                return;
            }
            if (result.profile) root.profilePath = result.profile;
            root.message = result.message;
        }
        function onOnlineChanged() {
            if (!root.backend.online) root.pending = 0;
            else root.send("status");
        }
    }
    spacing: 13

    Flow {
        Layout.fillWidth: true
        spacing: 8
        GlassButton { objectName: "inputApply"; text: "Apply input now"; primary: true; enabled: root.saved && !root.pending; onClicked: root.send("apply") }
        GlassButton { objectName: "inputWrite"; text: "Write profile"; enabled: root.saved && !root.pending; onClicked: root.send("write_profile") }
        GlassButton { objectName: "inputNext"; text: "Next layout"; enabled: !root.pending; onClicked: root.send("next_layout") }
        GlassButton { text: "Refresh"; compact: true; enabled: !root.pending; onClicked: root.send("status") }
    }
    Text {
        Layout.fillWidth: true
        text: root.error || root.message || "Reading keyboards…"
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
        color: root.error ? Theme.amber : Theme.inkDim
        font.pixelSize: 12
    }
    Text {
        Layout.fillWidth: true
        visible: !!root.profilePath
        text: root.profilePath
        textFormat: Text.PlainText
        wrapMode: Text.WrapAnywhere
        color: Theme.cyan
        font.family: Theme.fontMono
        font.pixelSize: 11
    }
    Repeater {
        model: root.keyboards
        Text {
            required property var modelData
            objectName: "inputKeyboard" + modelData.name
            Layout.fillWidth: true
            text: modelData.name + "  ·  " + modelData.active_keymap
            textFormat: Text.PlainText
            elide: Text.ElideRight
            color: modelData.main ? Theme.cyan : Theme.inkDim
            font.pixelSize: 12
        }
    }
    GridLayout {
        Layout.fillWidth: true
        columns: width < 490 ? 1 : 2
        columnSpacing: 12
        rowSpacing: 10
        SettingsField {
            Layout.fillWidth: true
            controlName: "inputLayouts"
            label: "XKB layouts (e.g. us,gb)"
            value: root.profile.layouts
            limit: 131
            onEdited: value => root.edited("layouts", value.trim())
        }
        SettingsField {
            Layout.fillWidth: true
            controlName: "inputVariants"
            label: "XKB variants (e.g. intl,)"
            value: root.profile.variants
            limit: 131
            onEdited: value => root.edited("variants", value.trim())
        }
        Repeater {
            model: [
                {key: "repeat_rate", label: "Repeat rate (Hz)", limit: 3},
                {key: "repeat_delay", label: "Repeat delay (ms)", limit: 4},
                {key: "pointer_speed", label: "Pointer speed (−1 to 1)", limit: 7}
            ]
            SettingsField {
                required property var modelData
                Layout.fillWidth: true
                controlName: "input_" + modelData.key
                label: modelData.label
                value: String(root.profile[modelData.key])
                limit: modelData.limit
                onEdited: value => root.edited(modelData.key, Number(value))
            }
        }
    }
    Repeater {
        model: [
            {key: "left_handed", label: "Left-handed mouse"},
            {key: "mouse_natural_scroll", label: "Natural mouse scrolling"},
            {key: "touchpad_natural_scroll", label: "Natural touchpad scrolling"},
            {key: "tap_to_click", label: "Tap to click"},
            {key: "disable_while_typing", label: "Disable touchpad while typing"},
            {key: "shortcut_bindings", label: "Include my Hyprlume shortcuts"},
            {key: "system_key_bindings", label: "Include system media, volume and brightness keys"}
        ]
        SettingsToggle {
            required property var modelData
            Layout.fillWidth: true
            controlName: "input_" + modelData.key
            label: modelData.label
            checked: root.profile[modelData.key]
            onToggled: value => root.edited(modelData.key, value)
        }
    }
}
