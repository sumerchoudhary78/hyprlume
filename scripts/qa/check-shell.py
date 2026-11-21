#!/usr/bin/env python3
"""Smoke the offline composition root on a private display, bus and audio server."""
import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inside-display', action='store_true')
    args = parser.parse_args()
    if not args.inside_display:
        with tempfile.TemporaryDirectory(prefix='hyprlume-shell-qa-') as runtime:
            env = os.environ.copy()
            for key in (
                'DISPLAY', 'WAYLAND_DISPLAY', 'HYPRLAND_INSTANCE_SIGNATURE', 'DBUS_SESSION_BUS_ADDRESS',
                'PIPEWIRE_REMOTE', 'PIPEWIRE_CONFIG_DIR', 'PIPEWIRE_CONFIG_NAME',
            ):
                env.pop(key, None)
            env.update(
                XDG_RUNTIME_DIR=runtime, XDG_CONFIG_HOME=runtime, XDG_STATE_HOME=runtime,
                PIPEWIRE_RUNTIME_DIR=runtime, HYPRLUME_SOCKET=str(Path(runtime) / 'offline.sock'),
                QT_QPA_PLATFORM='xcb', LIBGL_ALWAYS_SOFTWARE='1', HYPRLUME_PREVIEW='1', HYPRLUME_QA='1',
            )
            return subprocess.call(
                ['xvfb-run', '-a', 'dbus-run-session', '--', sys.executable, __file__, '--inside-display'],
                env=env, cwd=ROOT,
            )

    audio_config = ROOT / 'scripts/qa/fixtures/pipewire.conf'
    socket = Path(os.environ['PIPEWIRE_RUNTIME_DIR']) / 'pipewire-0'
    with subprocess.Popen(['pipewire', '-c', str(audio_config)], stdout=subprocess.DEVNULL) as audio:
        try:
            deadline = time.monotonic() + 5
            while not socket.exists():
                if audio.poll() is not None or time.monotonic() >= deadline:
                    raise AssertionError('private PipeWire fixture did not start')
                time.sleep(0.05)
            with subprocess.Popen(
                ['quickshell', '-p', str(ROOT / 'shell/shell.qml')],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
            ) as shell:
                try:
                    output = shell.communicate(timeout=4)[0]
                except subprocess.TimeoutExpired:
                    shell.terminate()
                    try:
                        output = shell.communicate(timeout=5)[0]
                    except subprocess.TimeoutExpired:
                        shell.kill()
                        shell.communicate()
                        raise
        finally:
            if audio.poll() is None:
                audio.terminate()
    assert 'HYPRLUME_LIVE_PREVIEW_READY' in output, output
    assert not re.search(
        r'ReferenceError:|TypeError:|Unable to assign|Cannot assign|Binding loop|Failed to load|Failed to deserialize|ERROR:',
        output,
    ), output
    print('LIVE SHELL PASS: production entry loads with an offline backend and private audio server')
    layout = subprocess.run(
        ['quickshell', '-p', str(ROOT / 'shell/bar-layout-test.qml')],
        capture_output=True, text=True, timeout=20,
    )
    output = layout.stdout + layout.stderr
    assert layout.returncode == 0 and 'HYPRLUME_BAR_LAYOUT_PASS' in output, output
    print('BAR LAYOUT PASS: recording, microphone and clock remain reachable across widths and scales')
    atlas = subprocess.run(['quickshell', '-p', str(ROOT/'shell/atlas-geometry-test.qml')],
                           capture_output=True, text=True, timeout=15)
    output = atlas.stdout + atlas.stderr
    assert atlas.returncode == 0 and 'HYPRLUME_ATLAS_GEOMETRY_PASS' in output, output
    print('ATLAS GEOMETRY PASS: scaled and rotated outputs, negative origins and clipped windows')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
