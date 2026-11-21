import QtQuick
import Quickshell
import qs.fixtures
import qs.modules.topbar
import qs.theme

ShellRoot {
    PreviewData { id: sample }
    FloatingWindow {
        id: window
        visible: true
        implicitWidth: 1600
        implicitHeight: 120
        readonly property var widths: [400, 440, 520, 539, 540, 555, 569, 580, 600, 640, 720, 758, 760, 780, 820, 950, 1600]
        readonly property var scales: [1, 1.2]
        property int step: 0

        BarContent {
            id: bar
            backend: sample.backend
            shellState: sample.state
            workspaceLabel: "Desktop application development on a wide output"
            clock: QtObject {
                property string time: "11:59 PM"
                property string date: "Donnerstag, 30. September"
            }
        }

        function collectButtons(item, buttons) {
            for (const child of item.children) {
                if (!child.visible) continue;
                if (child.objectName) {
                    const left = child.mapToItem(bar, 0, 0).x;
                    buttons.push({name: child.objectName, left: left, right: left + child.width});
                } else {
                    collectButtons(child, buttons);
                }
            }
        }

        function layoutFits() {
            const buttons = [];
            collectButtons(bar, buttons);
            buttons.sort((a, b) => a.left - b.left);
            let edge = 0;
            for (const button of buttons) {
                if (button.left < edge - 0.01 || button.right > bar.width + 0.01) {
                    console.error("Bar control overlaps or clips", button.name, bar.width, Theme.scale);
                    return false;
                }
                edge = button.right;
            }
            return buttons.map(button => button.name).join(",") ===
                "openOverview,openLauncher,quickCommand,openNotifications,stopRecording,activeMicrophone,quickSettings";
        }

        Timer {
            id: advance
            interval: 50
            repeat: true
            onTriggered: {
                if (window.step > 0 && !window.layoutFits()) {
                    Qt.exit(1);
                    return;
                }
                if (window.step === window.widths.length * window.scales.length) {
                    console.log("HYPRLUME_BAR_LAYOUT_PASS", window.step);
                    Qt.quit();
                    return;
                }
                Theme.scale = window.scales[Math.floor(window.step / window.widths.length)];
                bar.width = window.widths[window.step % window.widths.length];
                window.step++;
            }
        }
        Component.onCompleted: {
            sample.backend.snapshot = Object.assign({}, sample.backend.snapshot, {voice: "wake"});
            sample.backend.capture = Object.assign({}, sample.backend.capture, {recording: true});
            advance.start();
        }
    }
}
