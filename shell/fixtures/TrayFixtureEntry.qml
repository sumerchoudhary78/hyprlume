import QtQuick
import Quickshell

QtObject {
    id: root
    property string text: ""
    property bool isSeparator: false
    property bool enabled: true
    property string icon: ""
    property int buttonType: 0
    property int checkState: Qt.Unchecked
    property var entries: []
    readonly property bool hasChildren: entries.length > 0
    signal triggered()
    onTriggered: {
        if (buttonType) checkState = checkState === Qt.Checked ? Qt.Unchecked : Qt.Checked;
        if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"tray",action:"menu",text:text,checked:checkState === Qt.Checked}));
    }
}
