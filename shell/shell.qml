import QtQuick
import Quickshell

ShellRoot {
    id: root
    readonly property bool preview: Quickshell.env("HYPRLUME_PREVIEW") === "1"
    Backend { id: backend }
    Loader {
        Component.onCompleted: setSource(root.preview ? "PreviewWindow.qml" : "DesktopShell.qml", {backend: backend})
    }
}
