import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.components
import qs.modules.home

// Only the atlas controls on an empty workspace accept pointer input.
ShellWindow {
    id: root
    required property var workspaces
    required property var shellState
    readonly property var currentWorkspace: screen ? workspaces.activeOn(screen) : null
    surfaceName: "wallpaper"
    anchors { top: true; bottom: true; left: true; right: true }
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Background
    mask: Region {
        Region {
            readonly property rect targets: landing.item?.mapTargets ?? Qt.rect(0, 0, 0, 0)
            x: targets.x; y: targets.y; width: targets.width; height: targets.height
        }
        Region {
            readonly property rect targets: landing.item?.buttonTargets ?? Qt.rect(0, 0, 0, 0)
            x: targets.x; y: targets.y; width: targets.width; height: targets.height
        }
    }
    Wallpaper { anchors.fill: parent }
    Loader {
        id: landing
        anchors.fill: parent
        active: !!root.currentWorkspace && root.workspaces.countFor(root.currentWorkspace) === 0
            && !root.shellState.menu && !root.shellState.servicePanelOpen && !root.shellState.locked
        sourceComponent: AtlasLanding {
            workspaces: root.workspaces; shellState: root.shellState
            currentWorkspace: root.currentWorkspace; outputScreen: root.screen
        }
    }
}
