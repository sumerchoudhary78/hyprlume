# Notifications (development branch / unreleased)

The Rust daemon implements the freedesktop.org notification interface on the
session bus. The shell shows glass popups without taking keyboard focus and a
notification centre opened from the bar's bell or:

```sh
hyprlume notifications
hyprlume notifications --status
```

The status command only reads state. Escape or the close button dismisses the
centre. Opening it marks its current entries read; removing a history item or
clearing the list also closes any corresponding active notification.

## Session ownership

There is one `org.freedesktop.Notifications` owner per D-Bus session. Hyprlume
requests it with **DoNotQueue only**, without replacing the existing owner or
allowing itself to be replaced. If another server owns it, the centre reports
unavailable. It watches for that owner to release the name and can then acquire
it without a polling loop. It does not stop, reconfigure or uninstall dunst,
mako, Caelestia or another desktop's notification daemon.

When intentionally adopting Hyprlume notifications, configure your session so
only the notification server you want starts. This is a manual choice; the
package does not change existing startup configuration.

## Behavior and limits

- Supports `Notify`, `CloseNotification`, `GetCapabilities`,
  `GetServerInformation`, `ActionInvoked` and `NotificationClosed`.
- Advertises plain body text and actions. Markup, hyperlinks, images, custom
  app icons, sounds, inline replies and Wayland activation tokens are not
  implemented. Notification text always renders as `Text.PlainText`; it never
  starts a process, loads an app-supplied file/URL or enters an AI transcript.
- New normal alerts use a seven-second default timeout. Zero timeout,
  resident and critical alerts stay active until dismissal or client recall.
  Positive non-critical, non-resident timeouts are honored.
- Do Not Disturb hides popups, including critical ones, while keeping history.
  Disabling it does not replay old alerts. Critical alerts remain in the centre.
- At most three popups are shown; shorter screens show fewer. Other alerts
  remain in the centre. The newest 24 entries are retained in memory, with
  summary/body/action bounds that fit the 64 KiB UI frame limit. Overflow closes
  the oldest active entry with the standard undefined/reserved reason.
- History and Do Not Disturb last for this daemon session only. Nothing is
  written to disk and durable `persistence` is not advertised. Transient alerts
  leave no history when they close. Expired entries remain readable but their
  actions are disabled because their notification IDs are no longer active.
- IDs and replacements are bound to the sending D-Bus connection. An unrelated
  application cannot replace or recall another sender's notice. UI actions also
  carry a revision, so an old button cannot invoke a replaced notice's action.
  Action and close signals are directed to the owning connection.
- Losing the session bus disables old actions/popups and retains passive
  history. Restart the daemon after a complete session-bus restart.

## Architecture

`src/services/notifications/store.rs` owns pure lifecycle transitions and bounds.
`native.rs` owns the standard D-Bus interface and its name lease. `mod.rs` owns
the UI watch channel and one deadline-driven expiry task, which sleeps without
periodic wakeups when there are no timed notifications. Session ownership changes
also use a signal stream. The native interface holds a weak controller reference
and the lease is released on shutdown/cancellation.

`shell/modules/notifications/` contains the centre, shared cards, action buttons
and popups. The daemon's `notifications_subscribe` topic shares the existing
private socket with agent and connectivity topics, but subscribers opt in
individually. AI previews and their confirmation tokens survive panel switches.

Protocol reference: [Desktop Notifications Specification 1.3](https://specifications.freedesktop.org/notification/latest-single/).

## Isolated verification

The normal Rust tests create their own private D-Bus daemon with an explicit
address; they do not connect to the user's session. Tests cover real methods,
directed signals, non-replacement of an existing server, later acquisition,
stale actions, expiry, critical/transient/resident behavior, DND and bounded state.

```sh
cargo test --locked --all-targets
python3 scripts/qa/render-desktop.py --notifications --interact
python3 scripts/qa/render-desktop.py --notifications --output output/design/notifications.png
python3 scripts/qa/render-desktop.py --notification-popups --output output/design/popups.png
```

QML checks use explicit synthetic clients under a private Xvfb display. To
manually test actual desktop clients without replacing your active session's
notification server, run the daemon and clients together inside a separate
`dbus-run-session` with a separate `--socket`. Do not start a development daemon
against your normal session bus just to preview the UI.
