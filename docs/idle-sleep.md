# Idle, locking and sleep

Available in the v0.5.0 desktop preview.
Development and CI use isolated sessions; installing or enabling this policy is
an explicit user choice. No compositor, logind or running-session configuration
is rewritten.

## Controls

Open **Settings → Idle & sleep** to save the policy. It applies live without
restarting the daemon. All automatic stages start disabled as a group. The saved
defaults, used when you enable that group, are:

| Stage | Default |
| --- | --- |
| Dim | 180 seconds, down to 20% |
| Lock | 300 seconds |
| Screen off | 360 seconds |
| Suspend | Disabled |

Use 0 to disable an individual stage; enabled stages must be strictly increasing
and between 5 and 86400 seconds. Screen off and suspend still require a confirmed
secure lock if the independent lock timer is disabled. Screen off currently
requires Hyprland. Dimming supports kernel backlights, not DDC/CI monitors.

The power panel's **Presentation mode** pauses automatic timers until switched
off or the daemon exits. Owned screen recording also pauses timers. Application
Wayland idle inhibitors are handled by swayidle/the compositor. None of these
pauses bypasses locking for an explicitly confirmed Hyprlume suspend.

```sh
hyprlume idle                         # status only
hyprlume idle --presentation on       # transient timer pause
hyprlume idle --presentation off      # start a fresh idle countdown
```

Equivalent configuration:

```toml
[preferences.idle]
enabled = false
dim_after_seconds = 180
lock_after_seconds = 300
screen_off_after_seconds = 360
suspend_after_seconds = 0
dim_percent = 20
```

Install `swayidle` with logind support and `swaylock >= 1.6` with its system PAM
configuration. Use one idle policy owner per session; an existing hypridle or
swayidle instance can otherwise apply its own independent timers. Hyprlume does
not terminate or replace existing services. The Wayland/logind environment must
belong to the intended local graphical login. A missing provider or verified
login is reported as unavailable, without a privileged fallback.

## Sleep guarantee and limits

Both confirmed manual suspend and opt-in idle suspend wait for the compositor's
session-lock acknowledgment. A failed lock, changed login, active sleep inhibitor,
or authorization challenge cancels dispatch. The confirmed login is checked again
after locking. Suspend does not request an interactive authorization dialog behind
the lock screen. Restart, shutdown and logout retain their separate review flow.

While idle policy is enabled, the provider also handles logind's before-sleep,
after-resume and lock signals, even during presentation mode/recording. A
before-sleep callback requests locking and waits for readiness. **External sleep
is best effort:** logind's delay inhibitor expires at the system's
`InhibitDelayMaxSec`; it cannot guarantee that a slow/failed lock prevents a
sleep requested by another process, a lid policy, root or firmware. Configure
system lid behavior through logind; Hyprlume does not take ownership of the lid
switch or change system policy in this milestone. Test that policy separately
before relying on automatic lid-close protection.

Activity, policy changes, presentation mode, provider failure and resume revoke
queued idle actions. A sleep request already dispatched to logind cannot be
recalled. Lock preparation may finish after cancellation; it never unlocks a
session automatically.

Dimming never increases a lower brightness. Restoration binds to the same login,
device identity and exact raw dimmed value; a subsequent user adjustment or device
replacement is left alone. DPMS leases similarly restore only matching outputs
that this policy attempted to blank. Read-back checks establish compositor/driver
state, not proof that a physical panel illuminated. Restoration errors stay
visible for manual recovery. Brightness/display leases are in memory; a forcibly
killed daemon cannot run cleanup.

## Implementation and verification

- `src/services/idle`: bounded callback admission, per-process capabilities,
  generation cancellation and owned swayidle lifecycle. Status never contains
  the callback capability. Callbacks from a replaced process are rejected.
- `src/app/idle.rs`: serial idle effects and restoration; sleep hooks bypass
  that queue to avoid a PrepareForSleep/Suspend deadlock.
- `src/services/power/idle.rs`: conditional brightness leases and guarded sleep.
- `src/desktop/display.rs`: fixed classic/Lua DPMS actions with read-back checks.
- `shell/modules/settings/IdleSettings.qml`: the live policy editor.

Rust tests exercise lock failure/readiness, changed logins, auth challenges,
restoration ownership, cancelled epochs, stale callbacks and both Hyprland IPC
formats. UI tests use synthetic state and real input in Xvfb. Native CI uses a
new disposable Linux account and headless Sway to exercise actual idle/activity
events, presentation/recording inhibition and the existing PAM lock checks.
Tests never suspend the host. Physical suspend/resume, lid behavior, multi-seat
and driver qualification remain required before a release.

Upstream contracts: [swayidle events and delay limits](https://github.com/swaywm/swayidle/blob/master/swayidle.1.scd),
[Hyprland DPMS dispatchers](https://github.com/hyprwm/hyprland-wiki/blob/main/content/configuring/core/dispatchers.md).
