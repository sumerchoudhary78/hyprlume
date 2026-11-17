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
from cases import commands as command_case, connectivity as connectivity_case, notifications as notification_case, audio as audio_case, power as power_case, tray as tray_case, launcher as launcher_case, tools as tools_case, settings as settings_case, locking as locking_case

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
    parser.add_argument('--notifications', action='store_true', help='render or exercise the notification centre')
    parser.add_argument('--notification-popups', action='store_true', help='render synthetic notification popups')
    parser.add_argument('--audio', action='store_true', help='render or exercise sound and media with synthetic devices')
    parser.add_argument('--audio-state', choices=['normal', 'unavailable'], default='normal')
    parser.add_argument('--power', action='store_true', help='render or exercise power controls with synthetic services')
    parser.add_argument('--power-state', choices=['normal', 'unavailable', 'desktop', 'inhibited', 'review'], default='normal')
    parser.add_argument('--locking', action='store_true')
    parser.add_argument('--settings', action='store_true')
    parser.add_argument('--settings-tab',type=int,choices=range(7),default=0)
    parser.add_argument('--tools-capture',action='store_true')
    parser.add_argument('--tools', action='store_true')
    parser.add_argument('--launcher', action='store_true')
    parser.add_argument('--tray', action='store_true', help='render or exercise application tray fixtures')
    parser.add_argument('--tray-state', choices=['normal', 'empty'], default='normal')
    parser.add_argument('--effects', action='store_true', help='render isolated liquid materials and test their motion clock')
    parser.add_argument('--inside-display', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if sum([args.effects, args.connectivity, args.notifications, args.notification_popups, args.audio, args.power, args.tray, args.launcher, args.tools, args.settings, args.locking]) > 1:
        parser.error('choose one preview surface per check')
    if args.effects and args.interact:
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
    env.update(HYPRLUME_NOTIFICATIONS_PREVIEW='1' if args.notifications and not args.interact else '0',
               HYPRLUME_NOTIFICATION_POPUPS='1' if args.notification_popups else '0')
    env.update(HYPRLUME_AUDIO_PREVIEW='1' if args.audio and not args.interact else '0', HYPRLUME_AUDIO_STATE=args.audio_state)
    env.update(HYPRLUME_POWER_PREVIEW='1' if args.power and not args.interact else '0', HYPRLUME_POWER_STATE=args.power_state)
    env.update(HYPRLUME_SETTINGS_PREVIEW='1' if args.settings and not args.interact else '0',HYPRLUME_SETTINGS_TAB=str(args.settings_tab),HYPRLUME_CAPTURE_PREVIEW='1' if args.tools_capture else '0')
    env.update(HYPRLUME_TOOLS_PREVIEW='1' if args.tools and not args.interact else '0')
    env.update(HYPRLUME_LAUNCHER_PREVIEW='1' if args.launcher and not args.interact else '0')
    env.update(HYPRLUME_TRAY_PREVIEW='1' if args.tray and not args.interact else '0', HYPRLUME_TRAY_STATE=args.tray_state)
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
            if args.locking:
                locking_case.run(driver)
            elif args.settings:
                settings_case.run(driver)
            elif args.tools:
                tools_case.run(driver)
            elif args.launcher:
                launcher_case.run(driver)
            elif args.tray:
                tray_case.run(driver, args.tray_state)
            elif args.power:
                power_case.run(driver, args.power_state, args.height)
            elif args.audio:
                audio_case.run(driver)
            elif args.notifications:
                notification_case.run(driver)
            elif args.connectivity:
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
