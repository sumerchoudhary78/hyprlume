import QtQuick

// Shared native-object policy; menu handles never become shell commands or IDs.
Item {
    id: root
    property bool active: false
    property var items: []
    property var entries: []
    property var selected: null
    property var menuStack: []
    property string error: ""
    readonly property var currentMenu: menuStack.length ? menuStack[menuStack.length - 1].handle : null
    readonly property string menuTitle: menuStack.length ? menuStack[menuStack.length - 1].title : "Application menu"
    readonly property var visibleEntries: entries.slice(0, 64)
    function label(text, fallback) {
        return String(text || fallback || "").replace(/[\u0000-\u001f\u007f]/g, " ").slice(0, 160);
    }
    function iconSource(source) { const value = String(source || ""); return value.startsWith("image://") || value.startsWith("file://") ? value : ""; }
    function contains(item) { return !!item && items.indexOf(item) >= 0; }
    function open(item) {
        if (!contains(item)) { error = "This application left the tray."; return false; }
        selected = item; error = "";
        menuStack = item.hasMenu && item.menu ? [{handle:item.menu,title:label(item.title || item.id, "Application")}]: [];
    }
    function activate(item, secondary) {
        if (!contains(item)) { error = "This application left the tray."; return; }
        error = "";
        if (item.onlyMenu && !secondary) { open(item); return false; }
        else if (secondary) item.secondaryActivate();
        else item.activate();
        return !secondary;
    }
    function scroll(item, delta, horizontal) {
        if (contains(item) && Number.isFinite(delta)) item.scroll(Math.max(-1200, Math.min(1200, Math.round(delta))), horizontal);
    }
    function enter(entry) {
        if (entries.indexOf(entry) < 0 || !entry.enabled || !entry.hasChildren) return;
        if (menuStack.length >= 8) { error = "Menu nesting limit reached."; return; }
        menuStack = menuStack.concat([{handle:entry,title:label(entry.text, "Submenu")}]); error = "";
    }
    function back() { if (menuStack.length > 1) menuStack = menuStack.slice(0, -1); }
    function trigger(entry) {
        if (!entry || entries.indexOf(entry) < 0 || !entry.enabled || entry.isSeparator) { error = "This menu action is unavailable."; return; }
        error = "";
        if (entry.hasChildren) enter(entry);
        else entry.triggered();
    }
    function reset() { selected = null; menuStack = []; error = ""; }
    onActiveChanged: if (!active) reset()
    onItemsChanged: if (selected && !contains(selected)) reset()
}
