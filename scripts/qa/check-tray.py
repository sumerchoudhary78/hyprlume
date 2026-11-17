#!/usr/bin/env python3
"""Exercise the native StatusNotifier/DBusMenu adapters on a private bus/display."""
import argparse
import json
import os
from pathlib import Path
import re
import select
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inside-display', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not args.inside_display:
        with tempfile.TemporaryDirectory(prefix='hyprlume-tray-qa-') as directory:
            env = os.environ.copy()
            for key in ['WAYLAND_DISPLAY', 'HYPRLAND_INSTANCE_SIGNATURE', 'DISPLAY', 'DBUS_SESSION_BUS_ADDRESS']:
                env.pop(key, None)
            env.update(XDG_RUNTIME_DIR=directory, XDG_CONFIG_HOME=directory, XDG_STATE_HOME=directory,
                       QT_QPA_PLATFORM='xcb', LIBGL_ALWAYS_SOFTWARE='1', HYPRLUME_TRAY_RECORD=str(Path(directory)/'events.json'))
            return subprocess.call(['xvfb-run', '-a', 'dbus-run-session', '--', sys.executable, str(Path(__file__).resolve()), '--inside-display'], env=env, cwd=ROOT)
    fixture = subprocess.Popen([sys.executable, str(ROOT/'scripts/qa/fixtures/tray.py')], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        if not select.select([fixture.stdout], [], [], 5)[0] or fixture.stdout.readline().strip() != 'TRAY_READY':
            raise AssertionError('tray fixture did not start; install python-dbus-next')
        result = subprocess.run(['quickshell', '-p', str(ROOT/'shell/tray-test.qml')], cwd=ROOT,
                                text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=20)
        if result.returncode or re.search(r'(HYPRLUME_TRAY_FAIL|ReferenceError:|TypeError:|Unable to assign|Cannot assign|ERROR:)', result.stdout):
            raise AssertionError(result.stdout)
        assert 'HYPRLUME_TRAY_NATIVE_PASS' in result.stdout, result.stdout
        events = json.loads(Path(os.environ['HYPRLUME_TRAY_RECORD']).read_text())
        assert events == [['activate', None], ['secondary', None], ['scroll', [120, 'vertical']], ['menu', 1], ['menu', 3], ['menu', 4]], events
        print('TRAY NATIVE PASS: real StatusNotifier icon/actions and DBusMenu round trips on private services')
        return 0
    except (AssertionError, subprocess.TimeoutExpired) as error:
        print('TRAY CHECK FAILED: '+str(error), file=sys.stderr)
        return 1
    finally:
        fixture.terminate()
        try:
            fixture.wait(timeout=3)
        except subprocess.TimeoutExpired:
            fixture.kill(); fixture.wait()


if __name__ == '__main__':
    raise SystemExit(main())
