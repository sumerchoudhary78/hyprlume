import QtQuick

// Test-only discovery of transformed controls for real pointer/keyboard QA.
// Names and rectangles go to stdout, never through the agent socket.
Item {
    id: root
    required property Item targetItem
    required property var backend
    visible: false
    function collect(item, out) {
        if (item.objectName && item.visible) {
            const p = item.mapToItem(root.targetItem, item.width / 2, item.height / 2);
            const bounds = item.mapToItem(root.targetItem, 0, 0, item.width, item.height);
            const inside = bounds.x >= 0 && bounds.y >= 0
                && bounds.x + bounds.width <= root.targetItem.width
                && bounds.y + bounds.height <= root.targetItem.height;
            out[item.objectName] = {x:p.x,y:p.y,enabled:item.enabled,activeFocus:item.activeFocus,inside:inside};
        }
        for (const child of item.children) collect(child, out);
    }
    Connections {
        target: root.backend
        function onSnapshotChanged() { if (root.enabled) timer.restart(); }
    }
    Timer {
        id: timer
        interval: 120
        repeat: true
        running: root.enabled
        onTriggered: {
            const controls = {};
            root.collect(root.targetItem, controls);
            console.log("HYPRLUME_QA " + JSON.stringify({kind:"controls",controls:controls}));
        }
    }
    Component.onCompleted: { if (enabled) timer.start(); }
}
