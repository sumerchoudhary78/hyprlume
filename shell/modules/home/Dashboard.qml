import QtQuick
import qs.modules.scene
import qs.modules.topbar
import qs.theme

// Reusable composition, independent of a layer-shell window. Live desktop,
// ordinary-window preview and screenshot tests all render this same tree.
Item {
    id: root
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    required property var state
    property bool showTopBar: true
    property bool presentation: false
    property bool animationsActive: true
    readonly property bool compact: width < 1100 || height < 620
    readonly property real sceneScale: Math.min(width / world.width, height / world.height)
    Wallpaper { id: wallpaper; anchors.fill: parent }
    Item {
        id: world
        width: root.compact ? 1000 : 1680
        height: root.compact ? 680 : 945
        anchors.centerIn: parent
        scale: root.sceneScale
        BarContent {
            visible: root.showTopBar
            x: root.compact ? 0 : 320; y: -1
            width: root.compact ? 1000 : 1360; height: 61
            backend: root.backend; state: root.state
            backdrop: wallpaper; presentation: root.presentation
        }
        Rail {
            x: root.compact ? 20 : 42; y: root.compact ? 148 : 91
            width: 76; height: root.compact ? 460 : 494
            rotation: 2.5
            backend: root.backend; workspaces: root.workspaces; backdrop: wallpaper
            reducedEffects: root.state.reducedEffects
        }
        QuickActions {
            visible: !root.compact
            x: 166; y: 228; width: 285; height: 247; rotation: 4
            backend: root.backend; workspaces: root.workspaces; backdrop: wallpaper
            reducedEffects: root.state.reducedEffects
        }
        WorkspaceFlow {
            visible: !root.compact
            x: 151; y: 493; width: 283; height: 174; rotation: -5.2
            backend: root.backend; workspaces: root.workspaces; backdrop: wallpaper
            reducedEffects: root.state.reducedEffects
        }
        Stage {
            x: root.compact ? 119 : 455; y: root.compact ? 143 : 202
            width: root.compact ? 856 : 806; height: root.compact ? 475 : 426
            rotation: 0.4
            backend: root.backend; workspaces: root.workspaces; metrics: root.metrics; clock: root.clock
            mode: root.state.mode; backdrop: wallpaper; reducedEffects: root.state.reducedEffects
        }
        AgentPanel {
            visible: !root.compact
            x: 1291; y: 149; width: 340; height: 302; rotation: -1.7
            backend: root.backend; focused: root.state.mode === 2; backdrop: wallpaper
            reducedEffects: root.state.reducedEffects
        }
        WorkspacesPanel {
            visible: !root.compact
            x: 1289; y: 467; width: 342; height: 186; rotation: 2.7
            workspaces: root.workspaces; backdrop: wallpaper; reducedEffects: root.state.reducedEffects
        }
        Loader {
            active: !root.compact && !root.state.reducedEffects
            x: 290; y: 614; width: 1040; height: 263
            source: "../scene/Holograms.qml"
            onLoaded: {
                item.globeActivated.connect(() => root.state.mode = 5);
                item.cityActivated.connect(() => root.state.mode = 4);
            }
        }
        SystemPanel {
            visible: !root.compact
            x: 29; y: 704; width: 313; height: 210; rotation: -4.5
            metrics: root.metrics; backdrop: wallpaper
            animationsActive: root.animationsActive && !root.backend.snapshot.panel_open && !root.backend.connectivity.panel_open && !root.backend.notifications.panel_open && !root.backend.snapshot.desktop_panel
            secondaryColor: Theme.violet; glowColor: Theme.amber
            reducedEffects: root.state.reducedEffects
        }
        ActionRing {
            visible: !root.compact
            x: 585; y: 635; width: 505; height: 250
            backend: root.backend; workspaces: root.workspaces
            reducedEffects: root.state.reducedEffects
        }
        ContainerPanel {
            visible: !root.compact
            x: 1355; y: 736; width: 294; height: 164; rotation: 5.5
            metrics: root.metrics; backdrop: wallpaper
            secondaryColor: Theme.cyan; glowColor: Theme.amber
            reducedEffects: root.state.reducedEffects
        }
        Text {
            visible: root.presentation
            x: 24; y: world.height - 17
            text: "DESIGN PREVIEW  /  SAMPLE SESSION DATA"
            color: "#8aabc6"; font.family: Theme.fontMono; font.pixelSize: 8; font.letterSpacing: 1.6
        }
    }
}
