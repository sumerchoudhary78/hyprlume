import QtQuick
import QtQuick.Controls
import qs.theme

CheckBox {
    id: control
    spacing: 8
    padding: 0
    opacity: enabled ? 1 : 0.42
    indicator: Rectangle {
        implicitWidth: 16; implicitHeight: 16
        y: (control.height - height) / 2
        radius: 5
        color: control.checked ? "#4050bde0" : Theme.glassInset
        border.width: 1
        border.color: control.activeFocus ? Theme.ink : control.checked ? Theme.cyan : Theme.rim
        Icon { anchors.centerIn: parent; size: 11; name: "check"; color: Theme.cyan; visible: control.checked }
    }
    contentItem: Text {
        text: control.text; textFormat: Text.PlainText
        leftPadding: control.indicator.width + control.spacing
        verticalAlignment: Text.AlignVCenter
        color: Theme.inkDim; font.family: Theme.fontSans; font.pixelSize: 11
    }
}
