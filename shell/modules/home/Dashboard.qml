import QtQuick
import QtQuick.Layouts
import qs.components
import qs.modules.scene
import qs.modules.topbar
import qs.theme

// The layer-shell home and both ordinary-window previews use this layout.
Item {
    id: root
    required property var backend
    required property var workspaces
    required property var metrics
    required property var clock
    required property var shellState
    property bool showTopBar: true
    property bool presentation: false
    property real wallpaperHeight: height

    readonly property int gutter: Theme.px(width >= Theme.px(900) ? 36 : 20)
    readonly property bool wide: content.width >= Theme.px(820)
    readonly property bool condensed: !wide || height < Theme.px(730)
    readonly property bool opticalGlass: wide && !viewport.interactive && !shellState.reducedEffects
        && !Theme.highContrast && Theme.glassRefraction > 0 && GraphicsInfo.api !== GraphicsInfo.Software
    readonly property string workspaceName: workspaces.focused
        ? workspaces.label(workspaces.focused) : "Home"

    Loader {
        id: wallpaper
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: root.wallpaperHeight
        active: root.showTopBar || root.opticalGlass
        sourceComponent: Wallpaper {
            visible: root.showTopBar
            sampleOnly: !root.showTopBar
        }
    }

    Rectangle {
        anchors.fill: parent
        visible: wallpaper.item?.custom ?? (Theme.wallpaperPath !== "")
        color: "#ad070f16"
    }

    Loader {
        id: bar
        z: 1
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: item?.implicitHeight ?? 0
        active: root.showTopBar
        sourceComponent: BarContent {
            backend: root.backend
            clock: root.clock
        }
    }

    FocusScroll {
        id: viewport
        anchors.fill: parent
        anchors.topMargin: bar.height
        contentWidth: width
        contentHeight: Math.max(height, content.y + content.implicitHeight + root.gutter)

        ColumnLayout {
            id: content
            x: Math.round((viewport.width - width) / 2)
            y: root.condensed ? Theme.px(18)
                : Math.round(Math.min(Theme.px(134), Math.max(Theme.px(24), (viewport.height - Theme.px(578)) * 0.3)))
            width: Math.max(0, Math.min(Theme.px(1320), viewport.width - root.gutter * 2))
            spacing: Theme.px(root.condensed ? 14 : 22)

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.px(12)

                Text {
                    Layout.fillWidth: true
                    text: root.workspaceName
                    textFormat: Text.PlainText
                    color: Theme.ink
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.px(!root.condensed ? 60 : root.width >= Theme.px(520) ? 40 : 32)
                    font.weight: Font.DemiBold
                    font.letterSpacing: Theme.px(root.condensed ? -1 : -2)
                    elide: Text.ElideRight
                    Accessible.role: Accessible.Heading
                }

                Text {
                    visible: root.presentation
                    text: "Sample session data"
                    color: Theme.inkDim
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontCaption
                }
            }

            GridLayout {
                Layout.fillWidth: true
                columns: root.wide ? 2 : 1
                columnSpacing: Theme.px(20)
                rowSpacing: Theme.px(16)

                Stage {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                    Layout.alignment: Qt.AlignTop
                    backend: root.backend
                    condensed: root.condensed
                    reducedEffects: root.shellState.reducedEffects
                    backdrop: root.opticalGlass ? wallpaper.item : null
                }

                ColumnLayout {
                    Layout.fillWidth: !root.wide
                    Layout.preferredWidth: root.wide ? Math.min(Theme.px(398), content.width * 0.36) : -1
                    Layout.minimumWidth: 0
                    Layout.alignment: Qt.AlignTop
                    spacing: Theme.px(16)

                    WorkspacesPanel {
                        Layout.fillWidth: true
                        workspaces: root.workspaces
                    }

                    SystemPanel {
                        Layout.fillWidth: true
                        visible: root.backend.settings.values.preferences.appearance.metrics
                        metrics: root.metrics
                    }
                }
            }
        }
    }
}
