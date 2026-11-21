# Desktop preferences

Open **Settings → Desktop**. The page groups default apps, appearance and accessibility, and language and time.

## Default apps and file associations

Choose HTTPS, HTTP, email, file manager, PDF or text, or enter another MIME type. Choose a registered app or enter an installed desktop-entry ID. **Set default** writes the association through GIO and checks the resolved default. Configure both HTTP and HTTPS if you want the same browser for both. **File manager** controls the handler for `inode/directory`, which the removable-drive panel uses for **Open files**.

See the [XDG default application specification](https://specifications.freedesktop.org/mime-apps/latest/default) for how user and desktop-specific associations are resolved.

## Appearance and accessibility

- **Wallpaper** accepts a local PNG, JPEG or WebP. Browse for an image, then save to apply it to the desktop. **Restore default** selects the First Light wallpaper. Install `qt6-imageformats` for WebP support where Qt does not provide it.
- **High contrast** brightens shell text and borders and disables its glass refraction. **Reduced effects** and shell scale remain in **Appearance**. With the Hyprlume portal profile, compatible apps receive contrast, color-scheme, accent and reduced-motion preferences. See [Desktop portals](portals.md).
- **GTK theme** sets the user launch profile’s theme override, for example `Adwaita:dark`. Leave it blank to inherit the session theme. **Qt Fusion** selects Fusion for compatible Qt widgets. **Qt app scale** sets a scaling factor for new Qt applications.
- Install **Orca** for the screen reader. **Start Orca screen reader** launches its installed desktop entry. **Enable Qt accessibility for new apps** adds the Qt accessibility setting to the launch profile.

## Language, time and application profile

Enter an installed locale and an IANA time zone to override `LANG` and `TZ` for applications using the profile. The locale controls the shell clock’s formatting. **24-hour shell clock** takes effect when saved.

Save the preferences and select **Write app profile** to generate `$XDG_CONFIG_HOME/environment.d/80-hyprlume-desktop.conf` (normally under `~/.config`). The profile is read for new processes started by the systemd user service manager after sign-in. A session launcher outside that manager can source the generated assignments before starting applications. For example, a shell launcher can use:

```sh
set -a
. "${XDG_CONFIG_HOME:-$HOME/.config}/environment.d/80-hyprlume-desktop.conf"
set +a
```

Clear the GTK, locale and time-zone fields, choose the inherited Qt style and default app scale, and write the profile again to use session defaults. This profile controls user application launches. System clock, system locale and display-manager settings are configured by the distribution. See [systemd environment.d](https://www.freedesktop.org/software/systemd/man/latest/environment.d.html).
