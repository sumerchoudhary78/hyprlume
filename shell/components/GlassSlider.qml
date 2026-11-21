import QtQuick
import QtQuick.Controls
import qs.theme

Slider {
    id: root
    from: 0; to: 1; stepSize: 0.01
    implicitHeight: Theme.px(30)
    leftPadding: Theme.px(8); rightPadding: leftPadding
    opacity: enabled ? 1 : 0.4
    background: Rectangle {
        x: root.leftPadding; y: root.topPadding + root.availableHeight / 2 - height / 2
        width: root.availableWidth; height: Theme.px(5); radius: height / 2
        color: Theme.field; border.width: 1; border.color: Theme.rim
        Rectangle {
            width: root.visualPosition * parent.width; height: parent.height; radius: parent.radius
            color: Theme.cyan
        }
    }
    handle: Rectangle {
        x: root.leftPadding + root.visualPosition * (root.availableWidth - width)
        y: root.topPadding + root.availableHeight / 2 - height / 2
        width: Theme.px(16); height: width; radius: height / 2
        color: root.pressed ? Theme.cyan : Theme.ink
        border.width: root.visualFocus ? 3 : 1
        border.color: root.visualFocus ? Theme.focusRing : Theme.field
    }
}
