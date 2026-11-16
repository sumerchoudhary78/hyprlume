#!/usr/bin/env python3
"""Render the actual QML desktop, or exercise it with real X11 input.

Runs on Linux in a private Xvfb display and D-Bus session. The only backend is
PreviewData.qml: no desktop dispatch, microphone, speaker, model, or network.
Requires quickshell, Qt Quick 3D, xvfb-run, xauth, dbus-run-session; xdotool for
--interact. This verifies the QML/RHI path, not a physical Wayland compositor.
"""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

from ui_driver import UiDriver
from cases import commands as command_case, connectivity as connectivity_case

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'output/design/desktop.png')
    parser.add_argument('--width', type=int, default=1680)
    parser.add_argument('--height', type=int, default=945)
    parser.add_argument('--reduced-effects', action='store_true')
    parser.add_argument('--interact', action='store_true')
    parser.add_argument('--connectivity', action='store_true', help='render or exercise the Wi-Fi/Bluetooth panel')
    parser.add_argument('--connectivity-state', choices=['normal', 'unavailable', 'pairing', 'pin'], default='normal')
    parser.add_argument('--effects', action='store_true', help='render isolated liquid materials and test their motion clock')
    parser.add_argument('--inside-display', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.effects and (args.interact or args.connectivity):
        parser.error('--effects and --interact are separate checks')
    if min(args.width, args.height) < 400:
        parser.error('preview dimensions must be at least 400 pixels')
    if not args.inside_display:
        for command in ['xvfb-run', 'dbus-run-session', 'quickshell']:
            if not shutil.which(command):
                parser.error(f'missing tool: {command}')
        with tempfile.TemporaryDirectory(prefix='hyprlume-ui-qa-') as runtime:
            env = os.environ.copy()
            for key in ['WAYLAND_DISPLAY', 'HYPRLAND_INSTANCE_SIGNATURE', 'DISPLAY', 'DBUS_SESSION_BUS_ADDRESS']:
                env.pop(key, None)
            env.update(XDG_RUNTIME_DIR=runtime, QT_QPA_PLATFORM='xcb', QSG_RHI_BACKEND='opengl', LIBGL_ALWAYS_SOFTWARE='1')
            command = ['xvfb-run', '-n', str(100 + os.getpid() % 30000), '-a', '-s', f'-screen 0 {max(args.width,1920)}x{max(args.height,1080)}x24',
                       'dbus-run-session', '--', sys.executable, str(Path(__file__).resolve()), *sys.argv[1:], '--inside-display']
            return subprocess.call(command, env=env, cwd=ROOT)
    subprocess.run([sys.executable, str(ROOT/'scripts/build-shaders.py'), '--check'], check=True)
    args.output = args.output.resolve()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(HYPRLUME_PREVIEW_WIDTH=str(args.width), HYPRLUME_PREVIEW_HEIGHT=str(args.height),
               HYPRLUME_REDUCED_EFFECTS='1' if args.reduced_effects else '0',
               HYPRLUME_SCREENSHOT='' if args.interact else str(args.output),
               HYPRLUME_QA='1' if args.interact else '0',
               HYPRLUME_EFFECTS_QA='1' if args.effects else '0',
               HYPRLUME_CONNECTIVITY_PREVIEW='1' if args.connectivity and not args.interact else '0',
               HYPRLUME_CONNECTIVITY_STATE=args.connectivity_state)
    proc = subprocess.Popen(['quickshell', '-p', str(ROOT/'shell/design-preview.qml')], cwd=ROOT, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    driver = UiDriver(proc)
    try:
        if not args.interact:
            code = proc.wait(timeout=40)
            driver.thread.join(timeout=2)
            if code != 0 or not args.output.exists() or 'HYPRLUME_RENDER_PASS' not in ''.join(driver.logs):
                raise AssertionError('desktop screenshot failed')
            print(f'RENDER PASS {args.width}x{args.height}: {args.output}')
            if args.effects:
                result = next((line.split('HYPRLUME_EFFECTS_RESULT ', 1)[1].strip() for line in driver.logs if 'HYPRLUME_EFFECTS_RESULT ' in line), None)
                if result is None:
                    raise AssertionError('liquid shader/motion checks did not complete')
                print('HYPRLUME_EFFECTS_RESULT '+result)
        else:
            if not shutil.which('xdotool'):
                raise AssertionError('xdotool is required for --interact')
            if args.connectivity:
                connectivity_case.run(driver)
            else:
                command_case.run(driver, args.reduced_effects)
        text = ''.join(driver.logs)
        if re.search(r'(ReferenceError:|TypeError:|Unable to assign|Cannot assign|Failed to load|Failed to deserialize|HYPRLUME_EFFECTS_FAIL|ERROR:)', text):
            raise AssertionError('runtime QML errors in log')
        return 0
    except (AssertionError, subprocess.SubprocessError, KeyError, IndexError) as error:
        print(f'FAIL: {error}\n'+''.join(driver.logs[-70:]), file=sys.stderr)
        return 1
    finally:
        driver.stop()



if __name__=='__main__':
    raise SystemExit(main())
