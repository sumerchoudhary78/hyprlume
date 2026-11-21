import QtQuick
import qs.modules.home
import qs.modules.topbar
import qs.theme

// Ordinary-window counterpart to the wallpaper, bar and on-demand Wayland layer.
Item {
    id: root
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    required property var shellState
    property bool presentation: false

    Wallpaper { id: backgroundImage; anchors.fill: parent }
    BarContent {
        id: bar
        anchors { left: parent.left; right: parent.right; top: parent.top }
        backend: root.backend; clock: root.clock; shellState: root.shellState
        workspaceLabel: root.workspaces.label(root.workspaces.focused)
    }
    Rectangle {
        anchors { left: parent.left; bottom: parent.bottom; margins: Theme.px(14) }
        width: sampleLabel.implicitWidth + Theme.px(18); height: sampleLabel.implicitHeight + Theme.px(8)
        radius: Theme.px(6); color: Theme.bar
        visible: root.presentation
        Text {
            id: sampleLabel
            anchors.centerIn: parent
            text: "Sample session"; font.family: Theme.fontSans; font.pixelSize: Theme.fontCaption; color: Theme.inkDim
        }
    }
    Loader {
        anchors { left: parent.left; right: parent.right; top: bar.bottom; bottom: parent.bottom }
        active: !!root.shellState.menu
        sourceComponent: DesktopMenu {
            backend: root.backend; shellState: root.shellState
            workspaces: root.workspaces; metrics: root.metrics; wallpaper: backgroundImage
        }
    }
}
