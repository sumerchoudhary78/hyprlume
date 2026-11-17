# Polkit authentication

Hyprlume can present Polkit authorization prompts for the active Wayland login.
The dialog shows the system action and the accounts that Polkit permits. It
follows PAM prompts, including additional factors and hidden input, and supports
retry and cancellation with Escape. This is a source feature for the next release.

## Start the agent

From the intended Wayland session with the Hyprlume daemon running, start it in a
terminal as the desktop user:

```sh
hyprlume polkit
```

For a source checkout use `./target/release/hyprlume polkit --path ./shell`.
Use the matching `--socket PATH` if the daemon uses a custom socket.

The desktop can use one primary Polkit agent per login. If another one is
registered, Hyprlume reports the conflict and exits. Select the agent you want
in your session's startup configuration before your next login.

For a systemd managed graphical session that imports its Wayland environment,
enable the optional service using the session's own terminal:

```sh
systemctl --user import-environment WAYLAND_DISPLAY XDG_SESSION_ID HYPRLAND_INSTANCE_SIGNATURE
systemctl --user enable hyprlume-polkit.service
```

It starts at the next activation of `graphical-session.target`. In a session
that uses compositor autostart instead, add `hyprlume polkit` to that startup
configuration after the daemon. Set `XDG_SESSION_ID` from the actual graphical
login when starting through a user manager.

Linux builds link GLib and the distribution's `libpolkit-agent-1`. Arch supplies
them in `glib2` and `polkit`. Source builds also need `pkgconf` and the GLib
development files. Ubuntu uses `libpolkit-agent-1-dev` for the build.

## Request and credential handling

The worker registers for a verified local logind session and accepts calls from
the registered Polkit authority's unique D-Bus sender. It admits one challenge
at a time. PAM sessions use the distribution's authentication helper, and the
agent completes the caller's request after the helper reports authorization.
System policy and the installed PAM stack determine which operations require
authentication and which factors they accept.

Quickshell renders the prompt on an exclusive keyboard layer. The dialog
sends responses over the worker's child process stdin. The daemon socket
carries only lock status to the dialog. The response is cleared from the
field after submission or cancellation, and the worker clears its Rust
response buffer on drop. The shell, daemon, model, and voice pipelines use
their existing processes and never receive PAM responses.

The agent cancels an active request when locking starts, when the lock
status connection breaks, or when the verified login changes. It also closes
after authority shutdown, explicit cancellation, three failed attempts or a
two minute request timeout. The service runs separately so the distribution's
Polkit helper can use its normal privileges while the AI daemon continues to
run with `NoNewPrivileges`. The normal shell and its design preview do not
register an agent.

## Verification

CI exercises the QML dialog with synthetic prompts and real input on a private
display. It separately runs the Rust worker against a real Polkit authority and
PAM in a disposable logind login. Native checks use `pkcheck` with a test action
to observe authorization without executing a privileged command. Real
multi-monitor Hyprland focus and the system's hardware PAM factors still need
testing on their target machines.

Upstream contract: [Polkit authentication agents](https://www.freedesktop.org/software/polkit/docs/latest/polkit-agents.html).
