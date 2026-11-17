"""Private StatusNotifier/DBusMenu app. Actions only write a test journal."""
import asyncio
import json
import os
from pathlib import Path

from dbus_next import Variant
from dbus_next.aio import MessageBus
from dbus_next.constants import PropertyAccess
from dbus_next.service import ServiceInterface, dbus_property, method, signal

EVENTS = []


def record(action, value=None):
    EVENTS.append([action, value])
    Path(os.environ['HYPRLUME_TRAY_RECORD']).write_text(json.dumps(EVENTS))


class Item(ServiceInterface):
    def __init__(self):
        super().__init__('org.kde.StatusNotifierItem')

    @dbus_property(access=PropertyAccess.READ)
    def Category(self) -> 's': return 'ApplicationStatus'

    @dbus_property(access=PropertyAccess.READ)
    def Id(self) -> 's': return 'hyprlume-qa'

    @dbus_property(access=PropertyAccess.READ)
    def Title(self) -> 's': return 'Hyprlume QA'

    @dbus_property(access=PropertyAccess.READ)
    def Status(self) -> 's': return 'Active'

    @dbus_property(access=PropertyAccess.READ)
    def WindowId(self) -> 'u': return 0

    @dbus_property(access=PropertyAccess.READ)
    def IconName(self) -> 's': return ''

    @dbus_property(access=PropertyAccess.READ)
    def IconPixmap(self) -> 'a(iiay)': return [[16, 16, bytes([255, 100, 180, 255])*256]]

    @dbus_property(access=PropertyAccess.READ)
    def IconThemePath(self) -> 's': return ''

    @dbus_property(access=PropertyAccess.READ)
    def OverlayIconName(self) -> 's': return ''

    @dbus_property(access=PropertyAccess.READ)
    def OverlayIconPixmap(self) -> 'a(iiay)': return []

    @dbus_property(access=PropertyAccess.READ)
    def AttentionIconName(self) -> 's': return ''

    @dbus_property(access=PropertyAccess.READ)
    def AttentionIconPixmap(self) -> 'a(iiay)': return []

    @dbus_property(access=PropertyAccess.READ)
    def AttentionMovieName(self) -> 's': return ''

    @dbus_property(access=PropertyAccess.READ)
    def ToolTip(self) -> '(sa(iiay)ss)': return ['', [], 'Fixture tray', 'Private protocol test']

    @dbus_property(access=PropertyAccess.READ)
    def ItemIsMenu(self) -> 'b': return False

    @dbus_property(access=PropertyAccess.READ)
    def Menu(self) -> 'o': return '/Menu'

    @method()
    def Activate(self, x: 'i', y: 'i'): record('activate')

    @method()
    def SecondaryActivate(self, x: 'i', y: 'i'): record('secondary')

    @method()
    def ContextMenu(self, x: 'i', y: 'i'): record('context')

    @method()
    def Scroll(self, delta: 'i', orientation: 's'): record('scroll', [delta, orientation])


class Menu(ServiceInterface):
    def __init__(self):
        super().__init__('com.canonical.dbusmenu')
        self.checked = False

    @dbus_property(access=PropertyAccess.READ)
    def Version(self) -> 'u': return 3

    @dbus_property(access=PropertyAccess.READ)
    def TextDirection(self) -> 's': return 'ltr'

    @dbus_property(access=PropertyAccess.READ)
    def Status(self) -> 's': return 'normal'

    @dbus_property(access=PropertyAccess.READ)
    def IconThemePath(self) -> 'as': return []

    def properties(self, item):
        props = {'enabled': Variant('b', item != 5), 'visible': Variant('b', True)}
        if item == 6:
            props['type'] = Variant('s', 'separator')
        else:
            props['label'] = Variant('s', {0: 'Root', 1: '&Open fixture', 2: 'Advanced', 3: 'Deep action', 4: 'Checked action', 5: 'Disabled'}[item])
        if item in (0, 2): props['children-display'] = Variant('s', 'submenu')
        if item == 4:
            props.update({'toggle-type': Variant('s', 'checkmark'), 'toggle-state': Variant('i', int(self.checked))})
        return props

    def layout(self, item, depth):
        ids = {0: [1, 6, 4, 2, 5], 2: [3]}.get(item, []) if depth != 0 else []
        children = [Variant('(ia{sv}av)', self.layout(child, depth-1 if depth > 0 else depth)) for child in ids]
        return [item, self.properties(item), children]

    @method()
    def GetLayout(self, parent: 'i', depth: 'i', names: 'as') -> 'u(ia{sv}av)':
        return [1, self.layout(parent, depth)]

    @method()
    def GetGroupProperties(self, ids: 'ai', names: 'as') -> 'a(ia{sv})':
        return [[item, self.properties(item)] for item in ids]

    @method()
    def AboutToShow(self, item: 'i') -> 'b': return False

    @method()
    def AboutToShowGroup(self, ids: 'ai') -> 'aiai': return [[], []]

    @method()
    def Event(self, item: 'i', event: 's', data: 'v', timestamp: 'u'):
        if event != 'clicked': return
        record('menu', item)
        if item == 4:
            self.checked = not self.checked
            self.ItemsPropertiesUpdated([[4, {'toggle-state': Variant('i', int(self.checked))}]], [])

    @signal()
    def ItemsPropertiesUpdated(self, updated: 'a(ia{sv})', removed: 'a(ias)') -> 'a(ia{sv})a(ias)':
        return [updated, removed]


async def main():
    bus = await MessageBus().connect()
    bus.export('/StatusNotifierItem', Item())
    bus.export('/Menu', Menu())
    await bus.request_name('org.hyprlume.TrayQA')
    print('TRAY_READY', flush=True)
    for _ in range(100):
        try:
            tree = await bus.introspect('org.kde.StatusNotifierWatcher', '/StatusNotifierWatcher')
            watcher = bus.get_proxy_object('org.kde.StatusNotifierWatcher', '/StatusNotifierWatcher', tree).get_interface('org.kde.StatusNotifierWatcher')
            await watcher.call_register_status_notifier_item('/StatusNotifierItem')
            break
        except Exception:
            await asyncio.sleep(0.1)
    else:
        raise RuntimeError('private tray watcher did not appear')
    await asyncio.Future()


if __name__ == '__main__':
    asyncio.run(main())
