import QtQuick
import Quickshell
import qs.services

TrayController {
    id: root
    items: Quickshell.env("HYPRLUME_TRAY_STATE") === "empty" ? [] : [sync, media]
    entries: currentMenu ? currentMenu.entries : []
    TrayFixtureEntry { id: open; text: "Open control centre" }
    TrayFixtureEntry { id: separator; isSeparator: true }
    TrayFixtureEntry { id: alerts; text: "Show alerts"; buttonType: 1; checkState: Qt.Checked }
    TrayFixtureEntry { id: details; text: "Connection details" }
    TrayFixtureEntry { id: advanced; text: "Advanced"; entries: [details] }
    TrayFixtureEntry { id: unavailable; text: "Unavailable action"; enabled: false }
    property QtObject menuRoot: QtObject { property var entries: [open, separator, alerts, advanced, unavailable] }
    property QtObject sync: QtObject {
        property string title: "Orbital Sync"
        property string icon: ""
        property int status: 1
        property bool hasMenu: true
        property bool onlyMenu: false
        property var menu: root.menuRoot
        function activate() { root.report("activate", title); }
        function secondaryActivate() { root.report("secondary", title); }
        function scroll(delta, horizontal) { root.report("scroll", delta); }
    }
    property QtObject media: QtObject {
        property string title: "Starfield media"
        property string icon: ""
        property int status: 2
        property bool hasMenu: true
        property bool onlyMenu: true
        property var menu: root.menuRoot
        function activate() { root.report("unexpected-activate", title); }
        function secondaryActivate() { root.report("secondary", title); }
        function scroll(delta, horizontal) { root.report("scroll", delta); }
    }
    function report(action, value) {
        if (Quickshell.env("HYPRLUME_QA") === "1") console.log("HYPRLUME_QA " + JSON.stringify({kind:"tray",action:action,value:value}));
    }
}
