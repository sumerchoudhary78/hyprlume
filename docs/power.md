# Power and session

Available in the v0.5.0 desktop preview.
Select the power button in Quick Settings or use the CLI.

```sh
hyprlume power           # open the panel; no settings are changed
hyprlume power --status  # read the latest service snapshot
```

The panel shares the shell's glass surfaces and liquid battery vessel. Battery
status, display brightness, power profiles and session actions work independently
when their corresponding services/hardware are available. Unsupported controls
remain disabled. A desktop without a system battery displays an AC-powered
state rather than an invented percentage.

## Supported behavior

- **Battery:** UPower's composite display device, charging/discharging state,
  finite percentages and the service's time estimate. This covers the system
  battery/UPS, rather than peripheral Bluetooth batteries. An unavailable or
  invalid reading remains unknown.
- **Brightness:** up to eight kernel backlights, 5–100%, using logind's
  session-scoped `SetBrightness`. No privileged helper, direct sysfs write or
  shell command is installed. Requests bind to the advertised device identity
  and range, rescan before writing, then read the driver's requested brightness
  back. Raw driver levels are rounded, so low-resolution devices can produce a
  nearby percentage. DDC/CI external-monitor and keyboard illumination controls
  are not implemented.
- **Profiles:** the profiles advertised by power-profiles-daemon: power saver,
  balanced and performance where available, with degraded-performance text.
  A selection is read back before success is reported. Hyprlume does not alter
  governors directly or create automatic profile rules.
- **Session:** suspend, log out, restart and shut down. Each first opens a
  separate review identifying the intended action and graphical login. The
  review lasts 30 seconds. Its token is single-use, and cancellation, panel
  closing, expiry, losing the last subscribed UI or a changed session prevents
  an old review from executing.

Restart/shutdown affect the whole computer. Logout terminates only the selected
logind session; it does not terminate every login or the user's service manager.
Save application work first. Suspend waits for a confirmed compositor lock and
revalidates the login before dispatch. A failed lock or authorization challenge
cancels sleep. See [idle/sleep behavior and external-sleep limits](idle-sleep.md).

Power requests are outside the AI planner. Their confirmations do not consume
or replace a pending agent confirmation. Escape cancels an open session review;
otherwise it closes the panel. The compact layout scrolls the same controls.

## Session identity and authorization

Native actions require an active, local Wayland/X11 user session owned by the
daemon's effective UID. SSH, inactive, greeter and other users' sessions are
rejected. If `XDG_SESSION_ID` is set, it must identify that graphical login; an
invalid hint does not fall through to another login. Without a hint, discovery
requires exactly one owned, active graphical session with a seat. An ambiguous
result disables session actions.

For systemd user services, import `XDG_SESSION_ID` from the intended desktop
before starting a matching development daemon. The setup guide's import line
includes it. Do not import an SSH session ID into the graphical user manager.
This implementation binds the unique logind owner, session object, UID and
creation timestamp, and revalidates them before mutation. A service restart
cannot retarget a reviewed logout to a new session.

Modern logind is required for the `*WithFlags` calls; systemd 257+ is the
supported baseline for inhibitor behavior. Hyprlume checks blocking inhibitors
and uses `ROOT_CHECK_INHIBITORS`, including weak blocks. It never requests an
inhibitor bypass. Reboot/shutdown authorization can use an existing polkit agent; Hyprlume
installs no policy rules and grants itself no privileges. A missing permission
or service error appears in the panel. Suspend requires authorization to be
already available; it never opens an interactive prompt behind a lock screen.

Once a confirmed request has been dispatched, closing the panel cannot undo
it. A D-Bus timeout does not prove the remote request was cancelled. Its token
is still consumed, and Hyprlume does not automatically retry uncertain actions.
Receipts for session actions say logind accepted the request, not that shutdown
or sleep has already completed.

## Code and idle work

```text
src/services/power/
├── types.rs             wire state and typed request/capability checks
├── mod.rs               admission, confirmation lifetime and state publication
├── native/
│   ├── mod.rs           system-bus events, observation and bounded file worker
│   ├── battery.rs       UPower display-device readings
│   ├── backlight.rs     bounded sysfs reads and logind brightness requests
│   ├── profiles.rs      native power-profiles-daemon adapter
│   └── session.rs       graphical-login binding, inhibitors and session methods
├── tests.rs             lifecycle, stale requests and concurrency regressions
└── bus_tests.rs         owned private D-Bus services and temporary backlights
shell/modules/power/    production glass controls and session review
```

The daemon publishes an independent `power` topic through its existing private
Unix socket. D-Bus signals update battery, profile and session observations;
service owner changes trigger a fresh snapshot. No periodic D-Bus polling runs
while the panel is hidden. Initial/event-driven snapshots also read backlight
state. A two-second file sampler runs only while Power is open so external
brightness changes and backlight hot-plug become visible; closing pauses it.
Losing the last power subscriber also pauses sampling, so a disconnected shell
does not leave a visibility-driven timer behind.
Only one blocking file worker per connection can be active, even if a driver read stalls.
Sysfs names and reads, view sizes and action admission are bounded.

Opening Power requests normal D-Bus activation for installed UPower/profile
services if they are not already running. It does not enable units for boot.
Hidden-panel startup only observes existing owners; it does not start these
optional services. Tests use a private bus with no service activation directories.

The UI debounces brightness input and sends a committed value after dragging or
keyboard input settles. There is no continuous brightness subprocess or audio
capture. Review countdown rendering exists only while a review is visible.

## Validation and limits

Rust tests create private D-Bus daemons and temporary backlight directories.
Fake logind methods record requests; they cannot suspend, terminate a login or
power off the host. Tests cover actual protocol/property round trips, read-back
brightness/profile observations, owner changes, inhibitors, remote/wrong-user/
ambiguous/replaced sessions, confirmation replay/expiry, delayed reads, busy
admission and error recovery. A hidden-panel regression checks that D-Bus reads
do not recur and that closing stops periodic brightness observation.

```sh
cargo test --locked --all-targets
python3 scripts/qa/render-desktop.py --power --interact
python3 scripts/qa/render-desktop.py --power --interact --width 960 --height 640 --reduced-effects
python3 scripts/qa/render-desktop.py --power --power-state unavailable --interact
python3 scripts/qa/render-desktop.py --power --power-state inhibited --interact
python3 scripts/qa/render-desktop.py --power --power-state review --output output/design/power-review.png
```

Native UI checks use only synthetic services inside a private Xvfb/D-Bus
session. The active Arch desktop, backlight, profiles and logins are untouched.
Physical brightness drivers, suspend/resume, polkit agents, logout/application
teardown, multiple seats and long-running performance still need qualification.
No release or AUR upgrade is part of this milestone.

API references: [logind](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html),
[UPower display device](https://upower.freedesktop.org/docs/UPower.html),
[power profiles](https://upower.pages.freedesktop.org/power-profiles-daemon/gdbus-org.freedesktop.UPower.PowerProfiles.html).
