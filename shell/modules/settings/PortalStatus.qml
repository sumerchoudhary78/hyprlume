import QtQuick
import QtQuick.Layouts
import qs.components
import qs.theme

ColumnLayout {
    id: root
    required property var backend
    property var snapshot: null
    property int pending: 0
    property string error: ""
    spacing: 16

    function refresh() {
        error = "";
        pending = backend.send({op: "portals_status"}) || 0;
        if (!pending) error = backend.error || "The desktop service is disconnected.";
    }

    Connections {
        target: root.backend
        function onResponse(id, ok, result, error) {
            if (id !== root.pending) return;
            root.pending = 0;
            if (ok) root.snapshot = result;
            else root.error = error;
        }
        function onOnlineChanged() {
            if (!root.backend.online && root.pending) {
                root.pending = 0;
                root.error = "The desktop service disconnected.";
            }
        }
    }
    Component.onCompleted: refresh()

    RowLayout {
        Layout.fillWidth: true
        Icon { name: "layers"; size: 20; color: Theme.cyan }
        Text {
            Layout.fillWidth: true
            text: "Desktop portals"
            color: Theme.ink
            font.family: Theme.fontSans
            font.pixelSize: 18
        }
        GlassButton {
            objectName: "refreshPortals"
            text: root.pending ? "Checking…" : "Refresh"
            iconName: "refresh"
            compact: true
            enabled: !root.pending
            onClicked: root.refresh()
        }
    }
    Text {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        textFormat: Text.PlainText
        text: root.error || root.snapshot?.message || "Checking the session bus…"
        color: root.error ? Theme.amber : Theme.inkDim
        font.family: Theme.fontSans
        font.pixelSize: 12
    }
    Text {
        Layout.fillWidth: true
        visible: !!root.snapshot
        wrapMode: Text.Wrap
        textFormat: Text.PlainText
        text: "Session desktop  " + (root.snapshot?.desktop || "Unspecified")
        color: Theme.inkFaint
        font.family: Theme.fontMono
        font.pixelSize: 11
    }
    GridLayout {
        Layout.fillWidth: true
        visible: !!root.snapshot
        columns: width < 420 ? 1 : 2
        columnSpacing: 10
        rowSpacing: 8
        Repeater {
            model: [
                {key: "frontend", label: "Portal service"},
                {key: "hyprland", label: "Hyprland backend"},
                {key: "gtk", label: "GTK backend"},
                {key: "appearance", label: "Hyprlume appearance"}
            ]
            Rectangle {
                id: tile
                required property var modelData
                readonly property string busState: root.snapshot?.[modelData.key] || "unknown"
                Layout.fillWidth: true
                Layout.preferredWidth: 190
                implicitHeight: 42
                radius: 13
                color: "#50304862"
                border.color: Theme.rim
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 10
                    Text {
                        Layout.fillWidth: true
                        text: tile.modelData.label
                        elide: Text.ElideRight
                        color: Theme.ink
                        font.family: Theme.fontSans
                        font.pixelSize: 12
                    }
                    Text {
                        text: ({running: "Running", on_demand: "On demand", missing: "Missing", unknown: "Unknown"})[tile.busState]
                        color: tile.busState === "running" ? Theme.mint : Theme.inkDim
                        font.family: Theme.fontMono
                        font.pixelSize: 10
                    }
                }
            }
        }
    }
    Text {
        visible: !!root.snapshot?.interfaces.length
        text: "App interfaces"
        color: Theme.inkDim
        font.family: Theme.fontSans
        font.pixelSize: 12
    }
    Repeater {
        model: root.snapshot?.interfaces || []
        RowLayout {
            required property var modelData
            required property int index
            objectName: "portalInterface" + index
            Layout.fillWidth: true
            Text {
                Layout.fillWidth: true
                text: modelData.label
                textFormat: Text.PlainText
                color: Theme.ink
                font.family: Theme.fontSans
                font.pixelSize: 12
            }
            Text {
                text: modelData.version === null ? "Unavailable" : "Version " + modelData.version
                color: modelData.version === null ? Theme.inkFaint : Theme.cyan
                font.family: Theme.fontMono
                font.pixelSize: 11
            }
        }
    }
}
