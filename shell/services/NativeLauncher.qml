import QtQuick
import Quickshell.Hyprland

LauncherController {
    id: root
    required property var backend
    property bool active: false
    NativeApplications { id: catalogue }
    apps: catalogue.apps
    windows: active && Hyprland.toplevels ? Hyprland.toplevels.values.map(window => ({
        handle:window, id:window.lastIpcObject.address || ("0x" + window.address.replace(/^0x/, "")),
        title:window.title || window.lastIpcObject.class || "Window",
        subtitle:(window.lastIpcObject.class || "Application") + " · Workspace " + (window.workspace ? window.workspace.name : "?"),
        icon:"", search:window.lastIpcObject.class || ""
    })) : []
    onRequest: operation => {
        pending = backend.send(operation) || 0;
        busy = pending > 0;
        if (!busy) error = "The desktop service is disconnected";
    }
    onActiveChanged: if (!active) { query = ""; error = ""; }
}
