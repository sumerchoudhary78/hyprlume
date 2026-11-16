import QtQuick
import QtQuick.Layouts
import Quickshell.Wayland
import qs.components
import qs.services
import qs.theme

// The desktop: a bottom-layer surface shown only when the focused workspace
// has no windows. It is not an overlay. With a window open it unmaps, so it
// costs nothing and never sits between the pointer and an application.
ShellWindow {
    id: root

    surfaceName: "home"

    required property Backend backend
    required property Workspaces workspaces
    required property Metrics metrics
    required property Clock clock
    required property ShellState state

    anchors {
        top: true
        bottom: true
        left: true
        right: true
    }

    WlrLayershell.layer: WlrLayer.Bottom
    visible: root.workspaces.desktopVisible

    readonly property bool showLeft: width >= Theme.px(1180)
    readonly property bool showRight: width >= Theme.px(1480)
    readonly property bool showRail: width >= Theme.px(880)
    readonly property bool showBand: height >= Theme.px(720) && width >= Theme.px(760)
    readonly property int leftWidth: Math.round(Math.max(Theme.px(258), Math.min(Theme.px(300), width * 0.17)))
    readonly property int rightWidth: Math.round(Math.max(Theme.px(288), Math.min(Theme.px(330), width * 0.19)))
    readonly property int bandHeight: Theme.px(206)

    Item {
        id: content

        anchors {
            fill: parent
            topMargin: Theme.barZone
            leftMargin: Theme.margin
            rightMargin: Theme.margin
            bottomMargin: Theme.margin
        }

        RowLayout {
            id: body

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: root.showBand
                ? parent.height - root.bandHeight - Theme.gap
                : parent.height
            spacing: Theme.gap

            Rail {
                visible: root.showRail
                Layout.fillHeight: true
                Layout.preferredWidth: Theme.railWidth
                workspaces: root.workspaces
            }

            ColumnLayout {
                id: leftColumn

                visible: root.showLeft
                Layout.fillHeight: true
                Layout.preferredWidth: root.leftWidth
                spacing: Theme.gap

                QuickActions {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: Theme.px(300)
                    backend: root.backend
                    workspaces: root.workspaces
                }

                WorkspaceFlow {
                    visible: leftColumn.height >= Theme.px(260)
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: Theme.px(260)
                    workspaces: root.workspaces
                }
            }

            Stage {
                Layout.fillWidth: true
                Layout.fillHeight: true
                backend: root.backend
                workspaces: root.workspaces
                metrics: root.metrics
                clock: root.clock
            }

            ColumnLayout {
                id: rightColumn

                visible: root.showRight
                Layout.fillHeight: true
                Layout.preferredWidth: root.rightWidth
                spacing: Theme.gap

                AgentPanel {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: Theme.px(392)
                    backend: root.backend
                    focused: root.state.mode === 2
                }

                WorkspacesPanel {
                    visible: rightColumn.height >= Theme.px(320)
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: Theme.px(300)
                    workspaces: root.workspaces
                }
            }
        }

        RowLayout {
            id: band

            visible: root.showBand
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: root.bandHeight
            spacing: Theme.gap

            SystemPanel {
                visible: root.showLeft
                Layout.fillHeight: true
                Layout.preferredWidth: root.leftWidth
                metrics: root.metrics
            }

            Item { Layout.fillWidth: true }

            ActionRing {
                Layout.fillHeight: true
                Layout.preferredWidth: Math.min(Theme.px(420), band.width * 0.34)
                backend: root.backend
                workspaces: root.workspaces
            }

            Item { Layout.fillWidth: true }

            ContainerPanel {
                visible: root.showRight
                Layout.fillHeight: true
                Layout.preferredWidth: root.rightWidth
                metrics: root.metrics
            }
        }
    }
}
