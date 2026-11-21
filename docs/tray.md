# Application tray

Open **Quick settings → App tray** or run `hyprlume tray` in a matching development build.
The glass panel shows active StatusNotifier application icons and their menus.
Passive items are hidden; attention items get a static badge.

Icons forward primary activation, middle-click secondary activation and wheel
input to their owning application. Right-click or **Menu** opens its DBusMenu.
The themed menu supports submenus, separators, enabled/disabled entries,
checkbox/radio state and updates from the application. Only current native
object handles can be activated. Menu labels are plain text, and the icon view
accepts local/native image sources rather than fetching web URLs.

Use Tab/Enter/Space, Up/Down, Home/End, Back and Escape for keyboard navigation.
Escape returns from a submenu, then closes the tray. Display is capped at 64
applications/entries per level and eight levels. Closing releases menu handles
and their native subscriptions; there is no periodic tray polling or blinking
animation. Quickshell owns StatusNotifier/DBusMenu integration; Hyprlume adds
QML policy/presentation and Rust panel coordination, with no custom C++.

`NativeTray.qml` keeps the root menu referenced while browsing a submenu, so
releasing the previous view cannot destroy the native subtree. The fixture uses
the same `TrayController.qml` and production widgets. Native protocol checks
create a private fake StatusNotifier app and exercise real icon loading,
activation, secondary action, scrolling, menu clicks, submenu loading, checkbox
updates and stale-target rejection. The active desktop's tray is not used.

```sh
python3 scripts/qa/check-tray.py  # requires python-dbus-next and native QA tools
python3 scripts/qa/render-desktop.py --tray --interact
python3 scripts/qa/render-desktop.py --tray --tray-state empty --interact
python3 scripts/qa/render-desktop.py --tray --width 960 --height 640 --reduced-effects
```

Legacy XEmbed icons and app-owned context menus without DBusMenu are outside
this milestone. Native action APIs do not supply completion receipts; errors
from applications remain in Quickshell's log. A broader physical Wayland/app
matrix remains to be qualified.
