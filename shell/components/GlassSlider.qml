import QtQuick
import QtQuick.Controls
import qs.theme

Slider {
    id: root
    from: 0; to: 1; stepSize: 0.01
    implicitHeight: 30
    leftPadding: 8; rightPadding: 8
    opacity: enabled ? 1 : 0.4
    background: Rectangle {
        x: root.leftPadding; y: root.topPadding + root.availableHeight / 2 - height / 2
        width: root.availableWidth; height: 7; radius: 4
        color: "#12243a"; border.width: 1; border.color: Theme.rim
        Rectangle {
            width: root.visualPosition * parent.width; height: parent.height; radius: 4
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0; color: Theme.violet }
                GradientStop { position: 1; color: Theme.cyan }
            }
        }
    }
    handle: Rectangle {
        x: root.leftPadding + root.visualPosition * (root.availableWidth - width)
        y: root.topPadding + root.availableHeight / 2 - height / 2
        width: 16; height: 16; radius: 8
        color: root.pressed ? Theme.cyan : "#dbf8ff"
        border.width: root.activeFocus ? 3 : 2
        border.color: root.activeFocus ? Theme.violet : "#46839b"
    }
}
