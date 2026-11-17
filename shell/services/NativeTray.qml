import QtQuick
import Quickshell
import Quickshell.Services.SystemTray

TrayController {
    id: root
    items: SystemTray.items.values.filter(item => item.status !== Status.Passive).slice(0, 64)
    entries: opener.children.values
    // Keep the root handle referenced while a submenu is displayed. Releasing
    // its last handle would destroy the native tree, including that submenu.
    QsMenuOpener { menu: root.active && root.selected ? root.selected.menu : null }
    QsMenuOpener { id: opener; menu: root.active ? root.currentMenu : null }
}
