#!/usr/bin/env python3
"""Run a real portal gateway and GTK file dialog on a private bus/display."""
import argparse
import asyncio
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import time

from dbus_next import Message, MessageType, Variant
from dbus_next.aio import MessageBus
from fixtures import portal_hyprland

ROOT = Path(__file__).resolve().parents[2]
DESKTOP = "/org/freedesktop/portal/desktop"
FRONTEND = "org.freedesktop.portal.Desktop"
BACKEND = "org.freedesktop.impl.portal.desktop.hyprlume"
APPEARANCE = "org.freedesktop.appearance"
PORTAL = "org.freedesktop.portal."
DBUS = "org.freedesktop.DBus"


def private_setup(binary, base):
    config = base / "config"
    runtime = base / "run"
    data = base / "data"
    descriptors = base / "portals"
    services = data / "dbus-1/services"
    for path in (config, runtime, descriptors, services):
        path.mkdir(mode=0o700, parents=True)
    artifacts = ROOT / "packaging/portals"
    for name in ("hyprlume-portals.conf", "hyprlume.portal"):
        shutil.copy2(artifacts / name, descriptors / name)
    shutil.copy2("/usr/share/xdg-desktop-portal/portals/gtk.portal", descriptors / "gtk.portal")
    (descriptors / "hyprland.portal").write_text(
        "[portal]\nDBusName=org.freedesktop.impl.portal.desktop.hyprland\n"
        "Interfaces=org.freedesktop.impl.portal.ScreenCast;org.freedesktop.impl.portal.Screenshot;"
        "org.freedesktop.impl.portal.GlobalShortcuts;\n"
    )
    activation = BACKEND + ".service"
    (services / activation).write_text(
        (artifacts / activation).read_text().replace("/usr/bin/hyprlume", shlex.quote(str(binary)))
    )
    env = os.environ.copy()
    for name in ("DBUS_SESSION_BUS_ADDRESS", "DISPLAY", "WAYLAND_DISPLAY", "HYPRLAND_INSTANCE_SIGNATURE"):
        env.pop(name, None)
    env.update(
        XDG_RUNTIME_DIR=str(runtime), XDG_CONFIG_HOME=str(config), XDG_CONFIG_DIRS=str(base / "config-defaults"),
        XDG_DATA_HOME=str(data), XDG_DATA_DIRS="/usr/share", XDG_STATE_HOME=str(base / "state"),
        XDG_CURRENT_DESKTOP="Hyprlume:Hyprland", XDG_DESKTOP_PORTAL_DIR=str(descriptors),
        HYPRLUME_SOCKET=str(runtime / "hyprlume/control.sock"), GTK_USE_PORTAL="0", GDK_BACKEND="x11",
        GSETTINGS_BACKEND="memory", LC_ALL="C.UTF-8",
    )
    return subprocess.call([
        "xvfb-run", "-a", "dbus-run-session", "--", sys.executable, __file__,
        "--binary", str(binary), "--inside", str(base),
    ], env=env)


async def call(bus, destination, interface, member, signature="", body=None, path=DESKTOP):
    reply = await asyncio.wait_for(bus.call(Message(
        destination=destination, path=path, interface=interface, member=member,
        signature=signature, body=body or [],
    )), timeout=8)
    assert reply.message_type != MessageType.ERROR, (interface, member, reply.error_name, reply.body)
    return reply.body


async def subscribe(bus, interface, member):
    await call(bus, DBUS, DBUS, "AddMatch", "s", [
        f"type='signal',sender='{FRONTEND}',interface='{interface}',member='{member}'"
    ], path="/org/freedesktop/DBus")


async def portal_request(bus, name, method, signature, args, token, interact=None, **options):
    request = PORTAL + "Request"
    sender = bus.unique_name[1:].replace(".", "_")
    handle = f"{DESKTOP}/request/{sender}/{token}"
    answer = asyncio.get_running_loop().create_future()

    def listen(message):
        if (message.message_type == MessageType.SIGNAL and message.interface == request
                and message.member == "Response" and message.path == handle and not answer.done()):
            answer.set_result(message.body)

    bus.add_message_handler(listen)
    try:
        result = await call(bus, FRONTEND, PORTAL + name, method, signature,
                            [*args, {"handle_token": Variant("s", token), **options}])
        assert result == [handle], result
        if interact:
            await asyncio.to_thread(interact)
        try:
            return await asyncio.wait_for(answer, 12)
        except TimeoutError as error:
            raise AssertionError(f"No Response signal for {name}.{method} ({token})") from error
    finally:
        bus.remove_message_handler(listen)


def respond_to_dialog(title, file=None):
    import pyatspi

    def find(node, name):
        if node.name == name and node.getState().contains(pyatspi.STATE_SHOWING):
            return node
        for child in node:
            match = find(child, name)
            if match is not None:
                return match
        return None

    target = file.name if file else "Cancel"
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        desktops = pyatspi.Registry.getDesktop(0)
        dialog = find(desktops, title)
        node = find(dialog, target) if dialog is not None else None
        if node is not None:
            x, y, width, height = node.queryComponent().getExtents(pyatspi.DESKTOP_COORDS)
            if width > 0 and height > 0:
                subprocess.run(["xdotool", "mousemove", str(x + width // 2), str(y + height // 2),
                                "click", "--repeat", "2" if file else "1", "--delay", "80", "1"],
                               check=True, timeout=4)
                return
        time.sleep(0.1)
    raise AssertionError(f"Could not find {target} in the GTK chooser")


async def rpc(operation, **fields):
    reader, writer = await asyncio.open_unix_connection(os.environ["HYPRLUME_SOCKET"])
    try:
        writer.write(json.dumps(dict(id=1, op=operation, **fields)).encode() + b"\n")
        await writer.drain()
        packet = json.loads(await asyncio.wait_for(reader.readline(), 4))
        assert "result" in packet, packet
        return packet["result"]
    finally:
        writer.close()
        await writer.wait_closed()


async def check(binary, base, children):
    bus = await MessageBus().connect()
    backend = await MessageBus().connect()
    events = []
    sender = bus.unique_name[1:].replace(".", "_")
    session_path = f"{DESKTOP}/session/{sender}/hyprlume_qa"
    await portal_hyprland.bind(backend, session_path, events)
    status = json.loads(subprocess.check_output([binary, "portals"], text=True, timeout=4))
    assert status["frontend"] == "on_demand" and not status["interfaces"], status
    assert status["appearance"] == "on_demand" and status["profile_requested"], status
    assert subprocess.check_output([binary, "portals", "--profile"], text=True) == (
        ROOT / "packaging/portals/hyprlume-portals.conf"
    ).read_text()

    await subscribe(bus, PORTAL + "Request", "Response")
    await subscribe(bus, PORTAL + "Settings", "SettingChanged")
    # This first call activates the packaged appearance backend through D-Bus.
    settings = PORTAL + "Settings"
    data, = await call(bus, FRONTEND, settings, "ReadAll", "as", [["org.freedesktop.*"]])
    appearance = data[APPEARANCE]
    assert set(appearance) == {"color-scheme", "accent-color", "reduced-motion"}, appearance
    assert appearance["color-scheme"].value == 1
    assert appearance["accent-color"].value == [92 / 255, 225 / 255, 1.0]
    assert await call(bus, FRONTEND, settings, "ReadAll", "as", [["org.unrelated.*"]]) == [{}]

    children.append(subprocess.Popen([binary, "daemon", "--demo"]))
    for _ in range(100):
        if Path(os.environ["HYPRLUME_SOCKET"]).is_socket():
            break
        await asyncio.sleep(0.05)
    assert Path(os.environ["HYPRLUME_SOCKET"]).is_socket()

    changed = {}
    complete = asyncio.Event()

    def setting_changed(message):
        if message.interface == settings and message.member == "SettingChanged":
            namespace, key, value = message.body
            if namespace == APPEARANCE:
                changed[key] = value.value
            if {"accent-color", "reduced-motion"} <= changed.keys():
                complete.set()

    bus.add_message_handler(setting_changed)
    saved = await rpc("settings_status")
    saved["values"]["preferences"]["appearance"].update(accent="mint", reduced_effects=True)
    await rpc("settings", request=dict(action="save", revision=saved["revision"], values=saved["values"]))
    await asyncio.wait_for(complete.wait(), 8)
    assert changed == {"accent-color": [104 / 255, 240 / 255, 177 / 255], "reduced-motion": 1}, changed
    mint, = await call(bus, FRONTEND, settings, "ReadOne", "ss", [APPEARANCE, "accent-color"])
    assert mint.value == changed["accent-color"]
    bus.remove_message_handler(setting_changed)

    fixture = base / "Portal fixture.txt"
    fixture.write_text("Only this temporary file is shared in the test.\n")
    title = "Hyprlume portal file fixture"
    code, result = await portal_request(bus, "FileChooser", "OpenFile", "ssa{sv}", ["", title], "file_open",
                                        lambda: respond_to_dialog(title, fixture),
                                        current_folder=Variant("ay", os.fsencode(base) + b"\0"))
    assert code == 0 and result["uris"].value == [fixture.as_uri()], (code, result)
    code, _ = await portal_request(bus, "FileChooser", "OpenFile", "ssa{sv}", ["", title], "file_cancel",
                                   lambda: respond_to_dialog(title))
    assert code == 1, code

    code, _ = await portal_request(bus, "Screenshot", "Screenshot", "sa{sv}", [""], "screenshot",
                                   interactive=Variant("b", True))
    assert code == 1
    code, result = await portal_request(bus, "ScreenCast", "CreateSession", "a{sv}", [], "cast_create",
                                        session_handle_token=Variant("s", "hyprlume_qa"))
    assert code == 0, result
    session = result["session_handle"].value
    code, _ = await portal_request(bus, "ScreenCast", "SelectSources", "oa{sv}", [session], "cast_select",
                                   types=Variant("u", 1))
    assert code == 0
    code, result = await portal_request(bus, "ScreenCast", "Start", "osa{sv}", [session, ""], "cast_start")
    assert code == 0 and result["streams"].value[0][0] == 42, result
    await call(bus, FRONTEND, PORTAL + "Session", "Close", path=session)
    for _ in range(50):
        if events and events[-1] == "close":
            break
        await asyncio.sleep(0.04)
    assert events == [("screenshot", True), "create", ("select", 1), "start", "close"], events
    status = json.loads(subprocess.check_output([binary, "portals"], text=True, timeout=4))
    assert status["frontend"] == status["hyprland"] == status["gtk"] == status["appearance"] == "running", status
    assert len(status["interfaces"]) == 5 and all(item["version"] for item in status["interfaces"]), status
    bus.disconnect()
    backend.disconnect()
    print("PORTALS PASS: app-facing gateway, live appearance, GTK file approval/cancel and Hyprland routing fixtures")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--inside", type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    binary = str(args.binary.resolve())
    if args.inside:
        children = []
        try:
            asyncio.run(asyncio.wait_for(check(binary, args.inside, children), 65))
        finally:
            for process in reversed(children):
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
        return 0
    with tempfile.TemporaryDirectory(prefix="hyprlume-portal-qa-") as directory:
        return private_setup(Path(binary), Path(directory))


if __name__ == "__main__":
    raise SystemExit(main())
