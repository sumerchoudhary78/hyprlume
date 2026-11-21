import QtQuick
import Quickshell
import Quickshell.Hyprland
import "../models/AtlasGeometry.js" as Atlas

// Read-only view of Hyprland workspaces plus the focus actions the shell needs.
// Counts come from the toplevel list rather than workspace IPC fields, which
// vary between Hyprland releases.
QtObject {
    id: root

    readonly property var all: Hyprland.workspaces ? Hyprland.workspaces.values : []
    readonly property var focused: Hyprland.focusedWorkspace ?? null
    property bool atlasVisible: false
    property var atlasWindows: []
    readonly property var frames: atlasVisible ? Hyprland.toplevels.values.filter(win => win.workspace).map(win => {
        const ipc = win.lastIpcObject || {};
        const bounds = boundsFor(win.monitor ?? win.workspace.monitor);
        const address = String(ipc.address || win.address || "").replace(/^0x/, "");
        return {workspace: win.workspace.name, id: /^[0-9a-f]+$/i.test(address) ? "0x" + address : "",
            app: ipc.class || ipc.initialClass || "", title: win.title || ipc.class || "Window",
            frame: ipc.hidden || ipc.mapped === false ? null : Atlas.windowFrame(bounds, ipc.at, ipc.size),
            active: win.activated, floating: !!ipc.floating};
    }).sort((a, b) => Number(a.floating) - Number(b.floating) || Number(a.active) - Number(b.active)) : []
    // Geometry snapshots with identical values must preserve focus and hover in the map.
    onFramesChanged: if (JSON.stringify(frames) !== JSON.stringify(atlasWindows)) atlasWindows = frames;

    property Timer geometryRefresh: Timer {
        interval: 1800; repeat: true; triggeredOnStart: true
        running: root.atlasVisible && Hyprland.toplevels.values.length > 0
        onTriggered: Hyprland.refreshToplevels()
    }

    function boundsFor(monitor) {
        return Atlas.outputBounds(monitor, monitor ? Quickshell.screens.find(screen => screen.name === monitor.name) : null);
    }

    function aspectFor(workspace) {
        const bounds = boundsFor(workspace?.monitor);
        return bounds ? bounds.w / bounds.h : 16 / 9;
    }

    function windowsFor(workspace) {
        return workspace ? atlasWindows.filter(win => win.workspace === workspace.name) : [];
    }

    // Workspace identity is the name. Hyprland 0.57 gives named workspaces no
    // numeric id, and Quickshell reports -1 for every one of them, so comparing
    // ids would treat all named workspaces as the same workspace.
    function sameWorkspace(a, b) {
        if (!a || !b)
            return false;
        return a.name === b.name;
    }

    function activeOn(screen) {
        const monitor = Hyprland.monitors.values.find(item => item.name === screen.name);
        return monitor?.activeWorkspace ?? null;
    }

    function label(workspace) {
        if (!workspace)
            return "";
        if (workspace.name && workspace.name.length > 0 && workspace.name !== String(workspace.id))
            return workspace.name;
        return "Workspace " + workspace.id;
    }

    function countFor(workspace) {
        if (!workspace || !Hyprland.toplevels)
            return 0;
        return Hyprland.toplevels.values.filter(t => t.workspace && sameWorkspace(t.workspace, workspace)).length;
    }

    // Hyprland 0.55+ can run from a Lua config, and in that mode the request
    // socket expects a Lua expression instead of the classic "workspace 2"
    // string. Branch on `usingLua` at call time; the property is false until
    // the Hyprland module initializes, which is also when dispatch starts
    // working. Named workspaces use `name:` in Lua and the plain name in
    // classic mode; their numeric id is negative and not a valid target.
    function luaString(value) {
        let text = String(value);
        text = text.split("\\").join("\\\\");
        text = text.split("\"").join("\\\"");
        return "\"" + text + "\"";
    }

    function focus(workspace) {
        if (!workspace)
            return;
        const name = workspace.name ?? "";
        const named = name.length > 0 && name !== String(workspace.id);
        const special = named && name.indexOf("special:") === 0;
        if (Hyprland.usingLua) {
            const target = special ? luaString(name)
                : (named ? luaString("name:" + name) : String(workspace.id));
            Hyprland.dispatch("hl.dsp.focus({ workspace = " + target + " })");
        } else {
            Hyprland.dispatch("workspace " + (named ? name : workspace.id));
        }
    }

    // Hyprland's `e+1` / `e-1` targets are handled by the classic `workspace`
    // dispatcher only. Lua's `hl.dsp.focus` treats the string as a workspace
    // selector, so "e+1" silently resolves to nothing. Relative ids (`+1`,
    // `-1`) work in both modes and match the user's own keybinds.
    function focusNext() {
        if (Hyprland.usingLua)
            Hyprland.dispatch("hl.dsp.focus({ workspace = \"+1\" })");
        else
            Hyprland.dispatch("workspace +1");
    }

    function focusPrevious() {
        if (Hyprland.usingLua)
            Hyprland.dispatch("hl.dsp.focus({ workspace = \"-1\" })");
        else
            Hyprland.dispatch("workspace -1");
    }
}
