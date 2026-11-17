import QtQuick
import Quickshell
import qs.services

ShellRoot {
    id: root
    property int stage: 0
    property int ticks: 0
    NativeTray { id: tray; active: true }
    FloatingWindow {
        visible: true; implicitWidth: 80; implicitHeight: 80
        Image { id: icon; anchors.fill: parent; source: tray.items.length ? tray.items[0].icon : ""; sourceSize: Qt.size(32,32) }
    }
    function check(condition, message) {
        if (!condition) { console.error("HYPRLUME_TRAY_FAIL " + message); Qt.exit(1); }
    }
    Timer {
        interval: 100; repeat: true; running: true
        onTriggered: {
            if (++root.ticks > 120) { console.error("HYPRLUME_TRAY_FAIL stage " + root.stage + ": " + tray.error); Qt.exit(1); return; }
            if (root.stage === 0) {
                if (tray.items.length !== 1 || icon.status !== Image.Ready) return;
                check(tray.items[0].title === "Hyprlume QA", "unexpected private item");
                tray.activate(tray.items[0], false);
                tray.activate(tray.items[0], true);
                tray.scroll(tray.items[0], 120, false);
                tray.open(tray.items[0]); root.stage = 1;
            } else if (root.stage === 1) {
                if (tray.entries.length < 5) return;
                const disabled = tray.entries.find(entry => entry.text === "Disabled");
                tray.trigger(disabled);
                check(tray.error.length > 0, "disabled entry accepted");
                tray.trigger(tray.entries.find(entry => entry.text.indexOf("Open fixture") >= 0));
                tray.enter(tray.entries.find(entry => entry.text === "Advanced")); root.stage = 2;
            } else if (root.stage === 2) {
                const leaf = tray.entries.find(entry => entry.text === "Deep action");
                if (!leaf) return;
                tray.trigger(leaf); tray.back(); root.stage = 3;
            } else if (root.stage === 3) {
                const checkbox = tray.entries.find(entry => entry.text === "Checked action");
                if (!checkbox) return;
                tray.trigger(checkbox); root.stage = 4;
            } else if (root.stage === 4) {
                const checkbox = tray.entries.find(entry => entry.text === "Checked action");
                if (!checkbox || checkbox.checkState !== Qt.Checked) return;
                const stale = tray.items[0]; tray.items = [];
                tray.activate(stale, false);
                check(tray.error.length > 0, "removed tray item accepted");
                check(tray.iconSource("https://example.invalid/icon.png") === "", "remote icon accepted");
                console.log("HYPRLUME_TRAY_NATIVE_PASS icon, activation, secondary, scroll, menu, submenu, check updates and stale targets");
                Qt.quit();
            }
        }
    }
}
