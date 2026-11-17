# Desktop portals

Hyprlume supplies a portal profile for Wayland apps and sandboxed apps. The
`xdg-desktop-portal` service talks to the backend responsible for the request.
Its normal file access and screen sharing prompts still control what an app
can see. This profile and the appearance backend are source features for the
next release.

| App request | Backend in the Hyprlume profile |
| --- | --- |
| Open, save, and select files or folders | GTK |
| Screen sharing, screenshot prompts, and app shortcuts | Hyprland |
| Appearance hints | Hyprlume |
| Other GTK-supported desktop services | GTK |

Hyprland's backend uses the session's compositor and PipeWire for sharing.
GTK provides its standard file dialogs. Hyprlume publishes a dark preference,
the selected accent, and the reduced-motion preference from **Reduced effects**
under `org.freedesktop.appearance`. The backend starts on demand. It follows
appearance changes on the existing daemon socket, with the saved configuration
as its initial fallback. Settings uses one backend in this profile so apps
receive a single set of appearance hints.

## Select the profile

Install `xdg-desktop-portal`, `xdg-desktop-portal-hyprland`, and
`xdg-desktop-portal-gtk`. Screen sharing also uses working PipeWire and
WirePlumber services. The Hyprlume package installs its own appearance backend
manifest and D-Bus activation files. The portal service starts the backend when
it needs it.

For a future Hyprlume session, have the session launcher export
`XDG_CURRENT_DESKTOP=Hyprlume:Hyprland` before the portal service or portal
clients start. From the intended graphical session, its startup can import
those variables into the activation environment:

```sh
dbus-update-activation-environment --systemd XDG_CURRENT_DESKTOP WAYLAND_DISPLAY HYPRLAND_INSTANCE_SIGNATURE
```

The packaged file is `/usr/share/xdg-desktop-portal/hyprlume-portals.conf`.
User and administrator portal preferences take precedence. Users who prefer
different providers can use the relevant desktop-specific override. The
gateway selects its profile when it starts. Set the desktop identity at the
next login so the compositor, application launches, and the activation
environment agree.

## Inspect and troubleshoot

Open **Settings → Portals** or read the same status from a terminal:

```sh
hyprlume portals
hyprlume portals --profile
```

The status distinguishes running, on-demand, and missing D-Bus services.
If the gateway is running, it reads the interface versions it advertises to
apps. The checks do not activate the gateway or ask for files or capture.
`profile_requested` reflects the local desktop identifier. The actual routing
can differ when a higher-priority configuration overrides it or when the
gateway started with a different activation environment. The second command
prints the packaged profile.

For routing and activation diagnostics in the graphical session, read the
gateway and backend logs:

```sh
journalctl --user -b -u xdg-desktop-portal.service -u xdg-desktop-portal-hyprland.service -u xdg-desktop-portal-gtk.service -u hyprlume-portal-settings.service
```

## Verification

The portal check starts a real gateway and GTK backend on a private bus and
display. It checks appearance activation and change signals through the public
Settings API, approves one temporary file via the actual chooser, and cancels
a second request. A controlled Hyprland backend fixture verifies screenshot
cancellation and screen share session creation, source selection, start, and
close. Physical Hyprland capture, PipeWire delivery, and browser screen sharing
are target-session validation.

References: [portal backend configuration](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html)
and [application appearance settings](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html).
