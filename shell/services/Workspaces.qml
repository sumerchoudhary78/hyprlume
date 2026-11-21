import QtQuick
import Quickshell
import Quickshell.Hyprland

// Read-only view of Hyprland workspaces plus the focus actions the shell needs.
// Counts come from the toplevel list rather than workspace IPC fields, which
// vary between Hyprland releases.
QtObject {
    id: root

    readonly property var all: Hyprland.workspaces ? Hyprland.workspaces.values : []
    readonly property var focused: Hyprland.focusedWorkspace ?? null

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
