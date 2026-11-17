#!/usr/bin/env python3
"""Check the real QML dialog over private pipes with a synthetic worker."""
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tempfile
from threading import Thread

from ui_driver import UiDriver

ROOT = Path(__file__).resolve().parents[2]
ANSWER = "Polkit-QA-only-123"


def fixture():
    def send(kind, **fields):
        print(json.dumps(dict(kind=kind, **fields)), flush=True)

    def read_until(kind, **fields):
        for line in sys.stdin:
            packet = json.loads(line)
            if packet.get("kind") == kind and all(packet.get(key) == value for key, value in fields.items()):
                return
        raise SystemExit(1)

    read_until("available", value=True)
    send("registered")
    send("open", id=1, action="org.hyprlume.qa.authenticate",
         message="Authentication is required to open this test device.",
         accounts=["Desktop user", "Test administrator"])
    read_until("start", id=1, account=1)
    send("prompt", id=1, prompt=1, label="Password:", echo=False)
    read_until("respond", id=1, prompt=1, response=ANSWER)
    send("notice", id=1, message="Enter your verification code.", error=False)
    send("prompt", id=1, prompt=2, label="Verification code:", echo=False)
    read_until("respond", id=1, prompt=2, response="654321")
    send("retry", id=1)
    read_until("cancel", id=1)
    send("close", id=1)
    send("open", id=2, action="org.hyprlume.qa.authenticate", message="Another test request.", accounts=["Desktop user"])
    read_until("start", id=2, account=0)
    send("prompt", id=2, prompt=1, label="Password:", echo=False)
    read_until("available", value=False)
    send("close", id=2)
    print("HYPRLUME_QA " + json.dumps({"kind": "polkit_fixture_pass"}), file=sys.stderr, flush=True)
    for _ in sys.stdin:
        pass


def check(runtime):
    path = runtime / "lock.sock"
    with socket.socket(socket.AF_UNIX) as server:
        server.bind(str(path))
        server.listen()
        server.settimeout(8)
        connections = []

        def accept():
            client, _ = server.accept()
            connections.append(client)
            with client.makefile("rb") as io:
                assert json.loads(io.readline()) == {"id": 1, "op": "lock_subscribe"}
            client.sendall(b'{"id":1,"result":{"active":false}}\n')

        thread = Thread(target=accept, daemon=True)
        thread.start()
        env = dict(os.environ, HYPRLUME_SOCKET=str(path), HYPRLUME_POLKIT_TEST_SCRIPT=str(Path(__file__).resolve()))
        proc = subprocess.Popen(["quickshell", "-p", str(ROOT / "shell/polkit-test.qml")], env=env,
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        driver = UiDriver(proc)
        try:
            driver.wait_controls("polkitAccount")
            window = driver.xdo("search", "--onlyvisible", "--name", "^Hyprlume Polkit Preview$").splitlines()[0]
            driver.xdo("windowfocus", "--sync", window)
            driver.click("polkitAccount")
            driver.xdo("key", "End", "Return")
            driver.click("polkitContinue")
            driver.wait_controls("polkitResponse")
            driver.xdo("type", "--clearmodifiers", ANSWER)
            output = ROOT / "output/design/polkit.png"
            output.parent.mkdir(parents=True, exist_ok=True)
            # Use the Xvfb capture from Pillow, as with the shell previews.
            from PIL import ImageGrab
            ImageGrab.grab().save(output)
            driver.xdo("key", "Return")
            driver.wait_event(lambda e: e["kind"] == "controls" and
                              e["controls"].get("polkitResponse", {}).get("activeFocus", False))
            driver.xdo("type", "--clearmodifiers", "654321")
            driver.xdo("key", "Return")
            driver.wait_controls("polkitResponse", False)
            driver.xdo("key", "Escape")
            driver.wait_controls("polkitResponse")
            driver.xdo("type", "--clearmodifiers", "discard-this")
            connections[0].sendall(b'{"event":"locking","state":{"active":true}}\n')
            driver.wait_event(lambda e: e["kind"] == "polkit_fixture_pass")
            driver.wait_controls("polkitResponse", False)
            logs = "".join(driver.logs)
            assert ANSWER not in logs and "discard-this" not in logs
            assert not re.search(r"ReferenceError:|TypeError:|Unable to assign|Binding loop|Failed to load|ERROR:", logs), logs[-4000:]
            print("POLKIT UI PASS: account choice, masked input, two PAM prompts, retry, Escape and lock cancellation")
        except Exception:
            print("".join(driver.logs[-30:]), file=sys.stderr)
            raise
        finally:
            driver.stop()
            for client in connections:
                client.close()


if __name__ == "__main__":
    if "--worker-fixture" in sys.argv:
        fixture()
    elif "--private-display" in sys.argv:
        check(Path(os.environ["XDG_RUNTIME_DIR"]))
    else:
        with tempfile.TemporaryDirectory(prefix="hyprlume-polkit-ui-") as directory:
            env = os.environ.copy()
            for name in ["DISPLAY", "WAYLAND_DISPLAY", "HYPRLAND_INSTANCE_SIGNATURE", "DBUS_SESSION_BUS_ADDRESS"]:
                env.pop(name, None)
            env.update(XDG_RUNTIME_DIR=directory, XDG_CONFIG_HOME=directory, QT_QPA_PLATFORM="xcb", LIBGL_ALWAYS_SOFTWARE="1")
            raise SystemExit(subprocess.call(
                ["xvfb-run", "-a", "dbus-run-session", "--", sys.executable, __file__, "--private-display"], env=env))
