import QtQuick
import QtQuick.Controls

Button {
    id: control
    property bool primary: false
    property bool compact: false
    implicitHeight: compact ? 34 : 42
    implicitWidth: Math.max(compact ? 34 : 80, label.implicitWidth + 28)
    padding: 12
    hoverEnabled: true
    Accessible.name: text
    opacity: enabled ? 1 : 0.4
    contentItem: Text {
        id: label
        text: control.text
        color: control.primary ? "#102622" : "#e5edf2"
        font.pixelSize: 13
        font.weight: Font.Medium
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }
    background: Rectangle {
        radius: 11
        color: control.primary ? (control.down ? "#79c7b6" : "#a5e3d2") : (control.hovered ? "#334354" : "#263343")
        border.width: control.activeFocus ? 2 : 1
        border.color: control.activeFocus ? "#a5e3d2" : "#455366"
    }
}
