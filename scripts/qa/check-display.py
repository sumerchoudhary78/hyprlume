#!/usr/bin/env python3
"""Check display rollback and persistence with a private compositor fixture."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time

WLR_RANDR = r'''
import json, os, pathlib, sys
path = pathlib.Path(os.environ["HYPRLUME_TEST_OUTPUTS"])
outputs = json.loads(path.read_text())
args = sys.argv[1:]
if args == ["--json"]:
    print(json.dumps(outputs))
else:
    while args:
        name, fmt, xy, scale = args[1], args[3], args[5], float(args[7])
        assert args[:8:2] == ["--output", "--mode", "--pos", "--scale"], args
        output = next(row for row in outputs if row["name"] == name)
        for mode in output["modes"]:
            mode["current"] = fmt == f'{mode["width"]}x{mode["height"]}@{mode["refresh"]:.3f}Hz'
        assert any(mode["current"] for mode in output["modes"])
        output["position"] = dict(zip(("x", "y"), map(int, xy.split(","))))
        output["scale"] = scale
        args = args[8:]
    scratch = path.with_suffix(".tmp")
    scratch.write_text(json.dumps(outputs))
    scratch.replace(path)
'''


def wait_for(query, timeout=5):
    stop = time.monotonic() + timeout
    while time.monotonic() < stop:
        value = query()
        if value:
            return value
        time.sleep(.06)
    raise AssertionError("Timed out waiting for the display worker")


def run(binary, root):
    runtime = root / "run"
    runtime.mkdir(mode=0o700)
    hyprland = runtime / "hypr" / "fixture" / ".socket.sock"
    hyprland.parent.mkdir(parents=True)
    hyprland.touch()
    outputs = root / "outputs.json"
    outputs.write_text(json.dumps([{
        "name": "eDP-1", "enabled": True, "make": "Fixture", "serial": "sample-1",
        "position": {"x": 0, "y": 0}, "scale": 1,
        "modes": [{"width": 1920, "height": 1080, "refresh": refresh, "current": refresh == 60}
                  for refresh in (60, 144)],
    }]))
    command = root / "wlr-randr"
    command.write_text(f"#!{sys.executable}\n" + WLR_RANDR)
    command.chmod(0o700)
    sock = runtime / "control.sock"
    path = root / "config" / "config.toml"
    env = os.environ | {
        "PATH": f'{root}:{os.environ["PATH"]}', "XDG_RUNTIME_DIR": str(runtime),
        "XDG_CONFIG_HOME": str(root / "config"), "XDG_DATA_HOME": str(root / "data"),
        "WAYLAND_DISPLAY": "fixture", "HYPRLAND_INSTANCE_SIGNATURE": "fixture",
        "HYPRLUME_SESSION_READY": "", "HYPRLUME_SYSTEMD_DAEMON": "",
        "HYPRLUME_TEST_OUTPUTS": str(outputs),
    }

    def call(op, **fields):
        with socket.socket(socket.AF_UNIX) as client:
            client.settimeout(4)
            client.connect(str(sock))
            client.sendall(json.dumps(dict(id=1, op=op, **fields)).encode() + b"\n")
            return json.loads(client.makefile("rb").readline())

    def status():
        return call("status")["result"]["displays"]

    def layout():
        current = json.loads(outputs.read_text())[0]
        return current["position"]["x"], current["scale"]

    with (root / "daemon.log").open("w") as log:
        daemon = subprocess.Popen(
            [binary, "--socket", str(sock), "--config", str(path), "daemon"],
            env=env, stderr=log, stdout=log, umask=0o077,
        )
    try:
        wait_for(sock.exists)
        call("panel", panel="settings", open=True)
        call("display", request={"action": "refresh"})
        wait_for(lambda: status()["outputs"])

        def preview(x, scale):
            info = status()
            change = dict(info["outputs"][0]["active"], x=x, scale=scale)
            reply = call("display", request={"action": "preview", "revision": info["revision"], "changes": [change]})
            assert reply["result"]["accepted"], reply
            result = wait_for(lambda: status()["preview"])
            assert layout() == (x, scale)
            return result["token"]

        file = path.with_name("display-layouts.json")
        attempt = preview(64, 1.25)
        call("display", request={"action": "keep", "token": attempt})
        wait_for(lambda: not status()["busy"])
        assert layout() == (64, 1.25)
        assert file.stat().st_mode & 0o777 == 0o600
        saved = file.read_bytes()
        assert "error" in call("display", request={"action": "keep", "token": attempt})

        attempt = preview(88, 1.5)
        call("display", request={"action": "revert", "token": attempt})
        wait_for(lambda: not status()["busy"])
        assert layout() == (64, 1.25) and file.read_bytes() == saved

        preview(100, 1)
        wait_for(lambda: not status()["busy"], timeout=20)
        assert layout() == (64, 1.25)

        preview(120, 1)
        daemon.kill()
        daemon.wait(timeout=5)
        wait_for(lambda: layout() == (64, 1.25))
        assert file.read_bytes() == saved
        print("DISPLAY PASS: readback, keep, stale token, revert, timeout and daemon-loss rollback")
    except BaseException:
        print((root / "daemon.log").read_text(), file=sys.stderr)
        raise
    finally:
        if daemon.poll() is None:
            daemon.terminate()
            daemon.wait(timeout=8)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="display-qa-") as directory:
        run(str(args.binary.resolve()), Path(directory))
