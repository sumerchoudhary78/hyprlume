-- Entry point for the selectable Hyprlume session.
hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })

local config = (os.getenv("XDG_CONFIG_HOME") or (os.getenv("HOME") .. "/.config")) .. "/hyprlume/"
local function include(name)
    local path = config .. name
    local handle = io.open(path, "r")
    if not handle then
        return
    end
    handle:close()
    dofile(path)
end

hl.bind("SUPER + Escape", hl.dsp.exec_cmd("hyprlume launcher"))
hl.bind("SUPER + SHIFT + E", hl.dsp.exec_cmd("uwsm stop"))
hl.bind("SUPER + SHIFT + Q", hl.dsp.window.close())
include("session.lua")
include("hyprlume-input.lua")

hl.on("hyprland.start", function()
    hl.exec_cmd("uwsm finalize HYPRLAND_INSTANCE_SIGNATURE HYPRLUME_SESSION_READY=1")
end)
