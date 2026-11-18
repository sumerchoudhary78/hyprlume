# Display settings

Open **Settings → Displays** in a Hyprland session with `wlr-randr` installed.
Select an active output from the layout map or its connector name. Choose a
reported resolution and refresh rate, set scale, or place the output using
horizontal and vertical coordinates. Positions are in logical desktop pixels.
Changes can be staged for multiple displays before previewing.

**Preview layout** applies the staged settings through Wayland output management
and checks the compositor's reported state. Choose **Keep layout** within 15
seconds. **Revert**, closing the view, locking, and timeout restore the previous
layout. A separate worker handles rollback if the daemon goes away. The packaged
daemon starts that worker in its own systemd user scope. The worker restores
settings only on surviving outputs whose connector and display identity match.

Kept layouts are saved in `display-layouts.json` beside the Hyprlume config.
The selectable [Hyprlume session](session.md) remembers up to eight connected
output sets and restores a matching saved layout at startup or when a dock or
monitor reconnects. In manually managed Hyprland sessions, previews apply to the
current compositor and saved layouts remain available to the Hyprlume session.
Disabled outputs appear in the list. Enable them through Hyprland before editing.
Advanced monitor features such as HDR, per-monitor color profiles and mirroring
are configured through the compositor.

The display test checks actual readback, explicit revert, timeout, daemon-loss
rollback and private persistence against a controlled output service. The Settings
preview exercises staging and confirmation. Driver, docking and visibility checks
on physical outputs are part of release qualification.
