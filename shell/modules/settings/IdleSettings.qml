import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var policy
    required property var status
    signal edited(string key, var value)
    spacing: 16
    Text { Layout.fillWidth: true; text: "Idle & sleep"; color: Theme.ink; font.family: Theme.fontSans; font.pixelSize: 18 }
    SettingsToggle {
        Layout.fillWidth: true; controlName: "settingsIdleEnabled"; label: "Automatic idle actions"
        description: "Dim, lock and sleep when you step away. Changes apply when saved."
        checked: root.policy.enabled; onToggled: value => root.edited("enabled", value)
    }
    Text {
        Layout.fillWidth: true; wrapMode: Text.Wrap
        text: "Use seconds; 0 disables a stage. Keep enabled stages in order. Screen off and suspend always wait for a confirmed lock. Presentation mode and recording pause these timers."
        color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 12
    }
    Repeater {
        model: [
            {key: "dim_after_seconds", label: "Dim after (seconds)"},
            {key: "lock_after_seconds", label: "Lock after (seconds)"},
            {key: "screen_off_after_seconds", label: "Screen off after (seconds)"},
            {key: "suspend_after_seconds", label: "Suspend after (seconds)"},
            {key: "dim_percent", label: "Dim brightness (5–80%)"}
        ]
        SettingsField {
            required property var modelData
            Layout.fillWidth: true; controlName: "settingsIdle" + modelData.key
            label: modelData.label; value: String(root.policy[modelData.key]); limit: 5
            onEdited: value => root.edited(modelData.key, Number(value))
        }
    }
    Text {
        Layout.fillWidth: true; textFormat: Text.PlainText; wrapMode: Text.Wrap
        text: root.status.error || root.status.message
        color: root.status.error ? Theme.amber : Theme.inkFaint; font.family: Theme.fontSans; font.pixelSize: 11
    }
}
