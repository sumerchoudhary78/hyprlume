import QtQuick
import QtQuick.Layouts
import qs.components
import qs.modules.scene
import qs.theme

FocusScope {
    id: root
    required property var backend
    required property var shellState
    required property var workspaces
    property var currentWorkspace: workspaces.focused
    required property var metrics
    property Item wallpaper: null
    property real wallpaperHeight: height
    readonly property bool overview: shellState.menu === "overview"
    readonly property bool optical: currentWorkspace !== null && workspaces.countFor(currentWorkspace) === 0 && !shellState.reducedEffects
        && !Theme.highContrast && Theme.glassRefraction > 0 && GraphicsInfo.api !== GraphicsInfo.Software
    readonly property int inset: Theme.px(8)
    readonly property int padding: Theme.px(18)
    focus: true
    Keys.onEscapePressed: root.shellState.closeMenu()
    Component.onCompleted: forceActiveFocus()

    // The preview can reuse its wallpaper. Wayland needs a same-window source
    // for sampling, and only while the sheet is over an empty workspace.
    Loader {
        id: sample
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: root.wallpaperHeight
        visible: false
        active: root.optical && !root.wallpaper
        sourceComponent: Wallpaper { sampleOnly: true }
    }
    Rectangle { anchors.fill: parent; visible: root.overview; color: "#65081112" }
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        onClicked: root.shellState.closeMenu()
        onWheel: event => event.accepted = true
    }
    GlassSurface {
        id: sheet
        x: root.overview ? Math.round((root.width - width) / 2) : root.width - width - root.inset
        y: root.overview ? Math.max(root.inset, Math.min(Theme.px(80), (root.height - height) / 3)) : root.inset
        width: Math.min(Theme.px(root.overview ? 804 : 398), root.width - root.inset * 2)
        height: Math.min(layout.implicitHeight + root.padding * 2, root.height - root.inset * 2)
        radius: Theme.radiusCard
        tintTop: root.overview ? "#223433" : Theme.glassTop
        tint: root.overview ? "#122223" : Theme.glass
        backdrop: root.optical ? (root.wallpaper ?? sample.item) : null
        reducedEffects: root.shellState.reducedEffects
        transform: PanelEntrance { reducedEffects: root.shellState.reducedEffects }
        Accessible.role: Accessible.Dialog
        Accessible.name: title.text
        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }

        ColumnLayout {
            id: layout
            anchors.fill: parent; anchors.margins: root.padding
            spacing: Theme.px(13)

            RowLayout {
                Layout.fillWidth: true
                Icon { visible: root.overview; name: "compass"; size: Theme.px(25); color: Theme.atlasCopper }
                Text {
                    id: title
                    Layout.fillWidth: true
                    text: root.overview ? "Workspace atlas" : "Quick settings"
                    color: root.overview ? Theme.atlasPaper : Theme.ink
                    font.family: root.overview ? Theme.fontAtlas : Theme.fontSans
                    font.pixelSize: root.overview ? Theme.px(42) : Theme.fontHeadline
                    font.weight: root.overview ? Font.Normal : Font.DemiBold
                    Accessible.role: Accessible.Heading
                }
                IconTile {
                    objectName: "closeDesktopMenu"
                    quiet: true; size: Theme.px(32); iconSize: Theme.px(16)
                    iconName: "close"; tooltip: "Close " + title.text.toLowerCase()
                    onClicked: root.shellState.closeMenu()
                }
            }
            FocusScroll {
                id: scroll
                Layout.fillWidth: true; Layout.fillHeight: true
                Layout.preferredHeight: body.implicitHeight
                contentWidth: width; contentHeight: body.implicitHeight
                Loader {
                    id: body
                    width: scroll.width - (scroll.interactive ? Theme.px(11) : 0)
                    sourceComponent: root.overview ? overviewContent : controlsContent
                }
            }
        }
    }
    Component {
        id: overviewContent
        ColumnLayout {
            spacing: Theme.px(14)
            WorkspacesPanel {
                Layout.fillWidth: true
                workspaces: root.workspaces
                currentWorkspace: root.currentWorkspace
                shellState: root.shellState
                onChosen: root.shellState.closeMenu()
            }
            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider }
            OverviewCommand { Layout.fillWidth: true; backend: root.backend }
            Rectangle {
                Layout.fillWidth: true; implicitHeight: 1; color: Theme.divider
                visible: system.visible
            }
            SystemPanel {
                id: system
                Layout.fillWidth: true
                metrics: root.metrics
                visible: root.backend.settings.values.preferences.appearance.metrics
            }
        }
    }
    Component {
        id: controlsContent
        QuickSettings { backend: root.backend; shellState: root.shellState }
    }
}
