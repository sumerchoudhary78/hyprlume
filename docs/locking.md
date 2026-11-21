# Secure session locking

`hyprlume lock` and the lock buttons in Quick Settings and the Power panel request a real
Wayland session lock. `hyprlume lock --wait` waits for the compositor's lock
acknowledgment, rather than merely for a process to start. A request can fail;
only a confirmed lock or the native lock screen establishes protection.

The authentication surface is **Swaylock**, using `ext-session-lock-v1` and the
system's PAM policy. The glass background is a packaged vector scene; the input
indicator, failed-attempt state and authentication belong to the lock client.
There is no QML imitation of a password prompt, daemon password field, AI unlock
command, unlock RPC or password in command-line/environment arguments.
Swaylock's SIGUSR1 unlock shortcut is blocked with GNU `env --block-signal`.
The worker requires Swaylock 1.6 or newer with `--ready-fd`. Some Effects
builds omit that readiness option even when their version meets the minimum.

Install Arch's `swaylock` and `librsvg` packages. The system package manager must
own `/usr/bin/swaylock` and `/etc/pam.d/swaylock`, without group/other write
access. Hyprlume reads but never modifies that PAM policy. PAM configuration and
account authentication remain the system administrator's responsibility. This
protects local session access; it does not sandbox already-running processes
of the same account or replace the compositor's security boundary.

## Independent lifetime and readiness

A dedicated `hyprlume-lock.service` transient user unit owns the Rust worker and
lock client independently of the desktop shell and daemon. It is a notify unit:
the worker forwards readiness only after reading Swaylock's compositor
acknowledgment. Crashes restart the worker; compositor locking remains
fail-closed while the authentication surface is recovered. The unit follows
`graphical-session.target` when the graphical session ends, not the daemon's
lifetime. Run one lock provider in a session.

The daemon retains `NoNewPrivileges=true`. The separate lock unit explicitly
uses the normal PAM execution environment (`NoNewPrivileges=no`), allowing the
system's password-checking helper to work. This does not grant the daemon or
worker root privileges. State changes are observed through systemd's D-Bus
signals, with reconnection on bus loss; there is no idle status subprocess loop.

A private 0700 directory at `$XDG_RUNTIME_DIR/hyprlume-lock` holds a 0600 episode
marker. It survives shell/daemon restarts and remains after abnormal failure.
A normal client exit ends the episode. The compositor remains authoritative
when its connection or graphical session ends. A stale marker conservatively
keeps desktop actions paused until recovery; there is no expiry-based unlock.
`lock-worker` is an internal systemd entry point, not a standalone UI command.

## Voice, capture and agent behavior

Before starting its lock unit, the daemon:

- Stops microphone capture and speech playback. An in-progress synthesis cannot
  start playback after the lock epoch changes.
- Stops the clipboard watcher and clears volatile history. This does not clear
  another application's current Wayland clipboard selection.
- Cancels capture selection or finalizes the owned recorder. Partial recordings
  can remain for recovery if the recorder fails to finalize.
- Cancels pending model work, revokes existing action confirmations, closes
  service panels and pauses system metrics/liquid motion. New desktop mutations
  are rejected while locking or recovery is active. Already-dispatched actions
  cannot be undone by locking.

Microphone and clipboard collection remain off after unlocking. A daemon started
while an episode exists also starts with collection paused. Use the service
controls to opt in again. Locking a demo daemon is refused; design-preview lock
buttons use synthetic data and never start a real lock client.

Power's Suspend action now waits for confirmed locking. See [idle/sleep policy](idle-sleep.md)
for opt-in automatic stages and the bounded, best-effort guarantee for external
sleep. No system lid policy or existing idle service is replaced automatically.

## Recovery

If the authentication surface crashes, the compositor continues protecting the
session and the independent unit tries to restore the client. Restarting the
shell or daemon does not provide an unlock path. If a renderer cannot recover,
use an authenticated TTY or SSH session to inspect:

```sh
systemctl --user status hyprlume-lock.service
journalctl --user -u hyprlume-lock.service
```

Retrying `hyprlume lock --wait` can recreate a missing unit; restarting the lock
unit can restore its renderer if it still exists. These operations request
another real lock/authentication surface. If recovery is impossible, end the
old graphical session from the authenticated TTY and start a fresh login. Clear
a stale Hyprlume episode marker only after the old compositor/session has ended;
removing a marker does not unlock a compositor and is not an authentication
method. No automatic bypass or insecure fullscreen fallback is provided.

## Qualification

Unit tests cover demo refusal, client version requirements, action-epoch
revocation, mutation gating and cancellation of a waiting model plan. The normal
UI interaction check exercises both lock buttons against a synthetic backend.

`check-wayland-services.py --ci-provision` is restricted to a disposable GitHub
Actions VM. It creates a new test-only Linux account and real systemd user
manager, starts private headless Sway, and runs the actual daemon with
NoNewPrivileges enabled. It exercises real data-control clipboard delivery,
PNG capture and silent video recording, then the dedicated lock unit with
system PAM. It checks failed authentication, signal resistance, daemon restart,
client crash/recovery, successful authentication and microphone/history-off
state. No existing account, desktop, display or PAM configuration is modified.
The temporary test credential is passed to the test input tool on stdin and is
not included in reports or artifacts.

Headless tests establish protocol, orchestration and authentication behavior.
Physical Hyprland multi-monitor output changes, GPU/driver failure, keyboard
layouts, fingerprint/remote PAM providers, and suspend/resume remain release
qualification work. The currently installed desktop is not replaced by these
checks.

Upstream references: [Swaylock options and readiness contract](https://github.com/swaywm/swaylock/blob/master/swaylock.1.scd),
[Wayland session-lock protocol](https://wayland.app/protocols/ext-session-lock-v1),
[GNU signal controls](https://www.gnu.org/software/coreutils/manual/html_node/env-invocation.html).
