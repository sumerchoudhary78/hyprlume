#!/usr/bin/env python3
"""Render the actual QML desktop, or exercise it with real X11 input.

Runs on Linux in a private Xvfb display and D-Bus session. The only backend is
PreviewData.qml: no desktop dispatch, microphone, speaker, model, or network.
Requires quickshell, Qt Quick 3D, xvfb-run, xauth, dbus-run-session; xdotool for
--interact. This verifies the QML/RHI path, not a physical Wayland compositor.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'output/design/desktop.png')
    parser.add_argument('--width', type=int, default=1680)
    parser.add_argument('--height', type=int, default=945)
    parser.add_argument('--reduced-effects', action='store_true')
    parser.add_argument('--interact', action='store_true')
    parser.add_argument('--inside-display', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
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
    args.output = args.output.resolve()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(HYPRLUME_PREVIEW_WIDTH=str(args.width), HYPRLUME_PREVIEW_HEIGHT=str(args.height),
               HYPRLUME_REDUCED_EFFECTS='1' if args.reduced_effects else '0',
               HYPRLUME_SCREENSHOT='' if args.interact else str(args.output),
               HYPRLUME_QA='1' if args.interact else '0')
    proc = subprocess.Popen(['quickshell', '-p', str(ROOT/'shell/design-preview.qml')], cwd=ROOT, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    events = queue.Queue()
    logs = []
    controls = {}

    def read_logs():
        for line in proc.stdout:
            logs.append(line)
            match = re.search(r'HYPRLUME_QA (\{.*\})', line)
            if match:
                events.put(json.loads(match.group(1)))
    thread = threading.Thread(target=read_logs, daemon=True)
    thread.start()

    def wait_event(predicate, timeout=6):
        end = time.monotonic()+timeout
        while time.monotonic()<end:
            if proc.poll() is not None:
                raise AssertionError('QML exited before the interaction completed')
            try:
                event = events.get(timeout=0.1)
            except queue.Empty:
                continue
            if event['kind'] == 'controls':
                controls.clear()
                controls.update(event['controls'])
            if predicate(event):
                return event
        raise AssertionError('timed out waiting for a UI event')

    def xdo(*parts):
        return subprocess.check_output(['xdotool', *map(str, parts)], text=True).strip()

    def wait_controls(name, present=True):
        wait_event(lambda e: e['kind']=='controls' and ((name in e['controls']) == present))

    def click(name):
        if name not in controls:
            raise AssertionError(f'control missing: {name}; found {list(controls)}')
        position = controls[name]
        if not position['enabled']:
            raise AssertionError(f'control is disabled: {name}')
        xdo('mousemove', '--sync', round(position['x']), round(position['y']), 'click', '1')

    def request(op):
        return wait_event(lambda e:e['kind']=='request' and e['operation']['op']==op)['operation']

    try:
        if not args.interact:
            code = proc.wait(timeout=40)
            thread.join(timeout=2)
            if code != 0 or not args.output.exists() or 'HYPRLUME_RENDER_PASS' not in ''.join(logs):
                raise AssertionError('desktop screenshot failed')
            print(f'RENDER PASS {args.width}x{args.height}: {args.output}')
        else:
            if not shutil.which('xdotool'):
                raise AssertionError('xdotool is required for --interact')
            wait_controls('quickCommand')
            window_id = xdo('search', '--onlyvisible', '--name', '^Hyprlume Design Preview$').splitlines()[0]
            xdo('windowfocus', '--sync', window_id)
            click('quickCommand'); request('toggle'); wait_controls('commandInput')
            xdo('type', '--clearmodifiers', '--delay', '25', 'workspace 2'); xdo('key', 'Return')
            r = request('command')
            assert r['text']=='workspace 2' and r['dry_run'] is True, r
            wait_event(lambda e:e['kind']=='controls' and 'commandInput' in e['controls'] and not e['controls']['commandInput']['enabled'] and 'confirmPlan' not in e['controls'])
            assert not controls['previewCommand']['enabled'] and not controls['runDirect']['enabled'], 'busy controls remained enabled'
            wait_controls('confirmPlan')
            assert not controls['commandInput']['enabled'], 'review allowed a replacement command'
            click('confirmPlan'); r=request('confirm'); assert r['token']==42, r
            xdo('key','Escape'); request('toggle'); wait_controls('commandInput',False)
            print('PASS keyboard entry previews; busy controls disable; review locks input; confirmation carries the token')
            click('quickCommand'); request('toggle'); wait_controls('commandInput')
            xdo('type', '--clearmodifiers', '--delay', '25', 'volume 30')
            wait_event(lambda e:e['kind']=='controls' and e['controls'].get('runDirect',{}).get('enabled',False))
            click('runDirect'); r=request('command'); assert r['text']=='volume 30' and r['dry_run'] is False,r
            wait_event(lambda e:e['kind']=='controls' and 'commandInput' in e['controls'] and e['controls']['commandInput']['enabled'])
            xdo('key','Escape'); request('toggle'); wait_controls('commandInput',False)
            print('PASS direct execution requires the explicit Run direct button')
            click('toggleVoice'); r=request('voice'); assert r['mode']=='wake',r
            wait_event(lambda e:e['kind']=='controls')
            click('toggleVoice'); r=request('voice'); assert r['mode']=='off',r
            wait_event(lambda e:e['kind']=='controls')
            print('PASS wake-word control sends wake/off; fixture never opens a microphone')
            click('workspaceRow2'); e=wait_event(lambda e:e['kind']=='workspace'); assert e['name']=='Operations',e
            click('mode4'); e=wait_event(lambda e:e['kind']=='mode'); assert e['mode']==4,e
            print('PASS transformed workspace rows and top-bar navigation receive real pointer input')
            if not args.reduced_effects:
                click('hologramGlobeRegion'); e=wait_event(lambda e:e['kind']=='mode'); assert e['mode']==5,e
                click('hologramCityRegion'); e=wait_event(lambda e:e['kind']=='mode'); assert e['mode']==4,e
                print('PASS real 3D globe and city route to network and container views')
            else:
                assert 'hologramGlobeRegion' not in controls and 'hologramCityRegion' not in controls
            click('agentVolume'); r=request('command'); assert r['text']=='volume 35' and r['dry_run'] is True,r
            wait_controls('cancelPlan'); click('cancelPlan'); request('cancel')
            xdo('key','Escape'); request('toggle'); wait_controls('commandInput',False)
            click('launcherterminal'); r=request('command'); assert r['text']=='launch terminal' and r['dry_run'] is True,r
            print('PASS agent shortcut previews; dismissal cancels; spatial terminal launcher previews an allowlisted app')
            print('INTERACTION PASS: desktop controls and confirmation flow')
        text = ''.join(logs)
        if re.search(r'(ReferenceError:|TypeError:|Unable to assign|Cannot assign|Failed to load|ERROR:)', text):
            raise AssertionError('runtime QML errors in log')
        return 0
    except (AssertionError, subprocess.SubprocessError, KeyError, IndexError) as error:
        print(f'FAIL: {error}\n'+''.join(logs[-70:]), file=sys.stderr)
        return 1
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill(); proc.wait()
        thread.join(timeout=2)


if __name__=='__main__':
    raise SystemExit(main())
