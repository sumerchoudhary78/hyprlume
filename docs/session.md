# Hyprlume session

The package provides a **Hyprlume** entry in your Wayland login menu. Select it
before signing in. From a local TTY, use `exec hyprlume-session`. The launcher
requires Hyprland 0.57 or later and UWSM 0.27 or later. A session already running
under the same account is detected before startup.

UWSM sets the desktop identity to `Hyprlume:Hyprland` and manages the graphical
session and XDG application autostart. When Hyprland is ready, it exports the
Wayland socket and instance signature and signals readiness. A gated autostart
entry starts `hyprlume-session.target`, which starts the daemon, Quickshell and
Polkit agent. The user service manager keeps one instance per unit, and the daemon
also guards its control socket. UWSM stops the session services and clears the
session activation environment on logout.

File dialogs and screen sharing use the [packaged portal profile](portals.md)
when the relevant backends are installed. Secret Service activation uses the
session's configured provider. Set up password-based keyring unlock with your
login manager's PAM stack as described in [credential storage](credentials.md).
The Polkit agent follows the authority's single-agent registration per login.

## Configuration and exit

The packaged Lua profile is separate from your usual Hyprland config. Add
custom Hyprland settings and bindings to `$XDG_CONFIG_HOME/hyprlume/session.lua`
(normally `~/.config/hyprlume/session.lua`). It also loads the optional managed
[`hyprlume-input.lua` profile](input.md). Existing Hyprland sessions can continue
to use the [individual user services](setup.md#user-services).

- **Super+Escape** opens the launcher.
- **Super+Shift+Q** closes the active window.
- **Super+Shift+E** ends the managed session. The power panel also offers logout.

Run `hyprlume-session --dry-run` from the intended login context to inspect
UWSM's launch plan. The test suite checks launcher guards and the user unit
graph. Login manager and hardware validation use a separate login.
