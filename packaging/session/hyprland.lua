-- Entry point for the selectable Hyprlume session.
hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })
-- The capture sheet must leave the compositor scene before grim or slurp starts.
hl.layer_rule({ match = { namespace = "^hyprlume-tools$" }, no_anim = true })

local config = (os.getenv("XDG_CONFIG_HOME") or (os.getenv("HOME") .. "/.config")) .. "/hyprlume/"
local function include(name, environment)
    local path = config .. name
    local handle = io.open(path, "r")
    if not handle then
        return false
    end
    handle:close()
    assert(loadfile(path, "t", environment or _ENV))()
    return true
end

local function bind_starter_keys()
    -- The input profile can own any of these chords. Record its declarations
    -- during configuration so each chord has only one action.
    local function chord_key(chord)
        local parts = {}
        for part in chord:upper():gmatch("[^+%s]+") do
            table.insert(parts, part)
        end
        table.sort(parts)
        return table.concat(parts, "+")
    end
    local configured = {}
    local profile_hl = setmetatable({
        bind = function(chord, ...)
            configured[chord_key(chord)] = true
            return hl.bind(chord, ...)
        end,
    }, { __index = hl })
    include("hyprlume-input.lua", setmetatable({ hl = profile_hl }, { __index = _ENV }))

    local function bind(chord, dispatcher)
        if not configured[chord_key(chord)] then
            hl.bind(chord, dispatcher)
        end
    end
    local function launch(chord, command)
        bind(chord, hl.dsp.exec_cmd(command))
    end

    launch("SUPER + T", [[
        for terminal in xdg-terminal-exec foot kitty alacritty wezterm konsole gnome-terminal xterm; do
            command -v "$terminal" >/dev/null && exec "$terminal"
        done
        exec hyprlume launcher
    ]])
    launch("SUPER + B", [[
        browser=$(xdg-mime query default x-scheme-handler/https)
        if [ -n "$browser" ]; then exec gtk-launch "$browser"; fi
        exec hyprlume launcher
    ]])
    launch("SUPER + E", "xdg-open \"$HOME\"")
    launch("SUPER + Tab", "hyprlume launcher --windows")
    bind("SUPER + Q", hl.dsp.window.close())
    for workspace = 1, 10 do
        local key = workspace % 10
        bind("SUPER + " .. key, hl.dsp.focus({ workspace = tostring(workspace) }))
        bind("SUPER + SHIFT + " .. key, hl.dsp.window.move({ workspace = tostring(workspace), follow = false }))
    end
end

hl.bind("SUPER + Escape", hl.dsp.exec_cmd("hyprlume launcher"))
hl.bind("SUPER + SHIFT + E", hl.dsp.exec_cmd("uwsm stop"))
hl.bind("SUPER + SHIFT + Q", hl.dsp.window.close())
if include("session.lua") then
    include("hyprlume-input.lua")
else
    bind_starter_keys()
end

hl.on("hyprland.start", function()
    hl.exec_cmd("uwsm finalize HYPRLAND_INSTANCE_SIGNATURE HYPRLUME_SESSION_READY=1")
end)
