import QtQuick
import QtQuick.Controls
import qs.theme

Button {
    id: control
    padding: 9
    font.family: Theme.fontSans; font.pixelSize: 11
    implicitWidth: Math.min(280, Math.max(62, contentItem.implicitWidth + 22))
    implicitHeight: 31
    opacity: enabled ? 1 : 0.42
    background: Rectangle {
        radius: 10; color: control.hovered ? "#60354b69" : "#4616263e"
        border.width: 1; border.color: control.activeFocus ? Theme.cyan : Theme.rim
    }
    contentItem: Text {
        text: control.text; textFormat: Text.PlainText
        color: Theme.ink; font: control.font
        horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
}
