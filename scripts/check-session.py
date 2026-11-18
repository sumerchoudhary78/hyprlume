#!/usr/bin/env python3
"""Check the selectable session entries and guard the service startup path."""
import configparser
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1] / "packaging/session"


def entry(name):
    parser = configparser.ConfigParser()
    parser.read(ROOT / name)
    return parser["Desktop Entry"]


def check():
    assert entry("hyprlume.desktop")["Exec"] == "/usr/bin/hyprlume-session"
    assert entry("hyprlume-services.desktop")["Exec"] == "/usr/bin/hyprlume-session --services"
    with tempfile.TemporaryDirectory(prefix="hyprlume-session-") as temporary:
        root = Path(temporary)
        trace = root / "calls"
        commands = {
            "loginctl": '''case "$1" in
    show-user) printf '%s\n' 'current existing' ;;
    show-session) printf '%s\n' "${FIXTURE_LOGIN_TYPE:-tty}" ;;
esac
''',
            "systemctl": '''case " $* " in
    *" is-active "*) exit "${FIXTURE_GRAPHICAL_ACTIVE:-1}" ;;
esac
printf 'systemctl %s\n' "$*" >> "$FIXTURE_SESSION_TRACE"
''',
            "uwsm": '''printf 'uwsm %s\n' "$*" >> "$FIXTURE_SESSION_TRACE"
printf '%s\n' "${UWSM_WAIT_VARNAMES:-}" >> "$FIXTURE_SESSION_TRACE"
''',
        }
        for name, contents in commands.items():
            path = root / name
            path.write_text("#!/bin/sh\n" + contents)
            path.chmod(0o700)
        env = {
            "PATH": f"{root}:{os.defpath}",
            "FIXTURE_SESSION_TRACE": str(trace),
            "XDG_SESSION_ID": "current",
        }

        def invoke(*args, success=True, **settings):
            trace.write_text("")
            result = subprocess.run(
                [str(ROOT / "hyprlume-session"), *args],
                env=env | settings, capture_output=True, text=True, check=False,
            )
            assert (result.returncode == 0) == success, result
            return trace.read_text()

        assert "uwsm start -n -e -D Hyprlume:Hyprland -N Hyprlume -- Hyprland -c " in invoke("--dry-run")
        assert "HYPRLUME_SESSION_READY HYPRLAND_INSTANCE_SIGNATURE" in invoke()
        assert invoke(success=False, WAYLAND_DISPLAY="existing") == ""
        assert invoke(success=False, FIXTURE_GRAPHICAL_ACTIVE="0") == ""
        assert invoke(success=False, FIXTURE_LOGIN_TYPE="wayland") == ""
        assert invoke("--services", XDG_CURRENT_DESKTOP="Hyprlume:Hyprland") == ""
        assert invoke("--services", XDG_CURRENT_DESKTOP="Hyprland", HYPRLUME_SESSION_READY="1") == ""
        assert invoke("--services", XDG_CURRENT_DESKTOP="Hyprlume:Hyprland", HYPRLUME_SESSION_READY="1",
                      WAYLAND_DISPLAY="fixture", HYPRLAND_INSTANCE_SIGNATURE="fixture") == (
            "systemctl --user --no-block start hyprlume-session.target\n"
        )
    print("SESSION PASS: login entry, unmanaged session protection and readiness-gated startup")


if __name__ == "__main__":
    check()
