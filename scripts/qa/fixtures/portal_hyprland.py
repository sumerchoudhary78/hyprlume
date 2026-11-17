"""Compositor-side test doubles used by the real XDG portal frontend."""
from dbus_next import Variant
from dbus_next.constants import PropertyAccess
from dbus_next.service import ServiceInterface, dbus_property, method

DESKTOP = "/org/freedesktop/portal/desktop"
NAME = "org.freedesktop.impl.portal.desktop.hyprland"
IMPL = "org.freedesktop.impl.portal."


class Screenshot(ServiceInterface):
    def __init__(self, calls):
        super().__init__(IMPL + "Screenshot")
        self.calls = calls

    @dbus_property(access=PropertyAccess.READ)
    def version(self) -> "u":
        return 2

    @method()
    def Screenshot(self, handle: "o", app_id: "s", parent: "s", options: "a{sv}") -> "ua{sv}":
        self.calls.append(("screenshot", options.get("interactive", Variant("b", False)).value))
        return [1, {}]


class Session(ServiceInterface):
    def __init__(self, calls):
        super().__init__(IMPL + "Session")
        self.calls = calls

    @dbus_property(access=PropertyAccess.READ)
    def version(self) -> "u":
        return 1

    @method()
    def Close(self):
        self.calls.append("close")


class ScreenCast(ServiceInterface):
    def __init__(self, session_path, calls):
        super().__init__(IMPL + "ScreenCast")
        self.session_path = session_path
        self.calls = calls

    @dbus_property(access=PropertyAccess.READ)
    def version(self) -> "u":
        return 4

    @dbus_property(access=PropertyAccess.READ)
    def AvailableSourceTypes(self) -> "u":
        return 1

    @dbus_property(access=PropertyAccess.READ)
    def AvailableCursorModes(self) -> "u":
        return 1

    @method()
    def CreateSession(self, handle: "o", session: "o", app_id: "s", options: "a{sv}") -> "ua{sv}":
        assert session == self.session_path, session
        self.calls.append("create")
        return [0, {}]

    @method()
    def SelectSources(self, handle: "o", session: "o", app_id: "s", options: "a{sv}") -> "ua{sv}":
        self.calls.append(("select", options["types"].value))
        return [0, {}]

    @method()
    def Start(self, handle: "o", session: "o", app_id: "s", parent: "s", options: "a{sv}") -> "ua{sv}":
        self.calls.append("start")
        return [0, {"streams": Variant("a(ua{sv})", [[42, {}]])}]


class GlobalShortcuts(ServiceInterface):
    def __init__(self):
        super().__init__(IMPL + "GlobalShortcuts")

    @dbus_property(access=PropertyAccess.READ)
    def version(self) -> "u":
        return 1


async def bind(bus, session_path, calls):
    bus.export(session_path, Session(calls))
    for interface in (Screenshot(calls), ScreenCast(session_path, calls), GlobalShortcuts()):
        bus.export(DESKTOP, interface)
    await bus.request_name(NAME)
