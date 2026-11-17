#!/usr/bin/env python3
"""Exercise real PipeWire/MPRIS adapters in an entirely private session.

Needs PipeWire, python-dbus-next and the native render tools. No hardware,
system PipeWire socket, real player, microphone or user's D-Bus is accessed.
"""
import argparse
import json
import os
from pathlib import Path
import re
import select
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]


def stop(process):
    if process and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inside-display', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not args.inside_display:
        with tempfile.TemporaryDirectory(prefix='hyprlume-audio-qa-') as directory:
            env = os.environ.copy()
            for key in ['WAYLAND_DISPLAY', 'HYPRLAND_INSTANCE_SIGNATURE', 'DISPLAY', 'DBUS_SESSION_BUS_ADDRESS',
                        'PIPEWIRE_REMOTE', 'PIPEWIRE_RUNTIME_DIR', 'PIPEWIRE_CONFIG_DIR', 'PIPEWIRE_CONFIG_NAME']:
                env.pop(key, None)
            env.update(XDG_RUNTIME_DIR=directory, PIPEWIRE_RUNTIME_DIR=directory,
                       XDG_CONFIG_HOME=directory, XDG_STATE_HOME=directory,
                       QT_QPA_PLATFORM='xcb', LIBGL_ALWAYS_SOFTWARE='1')
            return subprocess.call(['xvfb-run', '-a', 'dbus-run-session', '--', sys.executable,
                                    str(Path(__file__).resolve()), '--inside-display'], env=env, cwd=ROOT)
    processes = []
    with tempfile.TemporaryFile(mode='w+') as pipewire_log:
        try:
            audio = subprocess.Popen(['pipewire', '-c', str(ROOT/'scripts/qa/fixtures/pipewire.conf')],
                                     stdout=pipewire_log, stderr=subprocess.STDOUT)
            processes.append(audio)
            deadline = time.monotonic() + 5
            while not (Path(os.environ['XDG_RUNTIME_DIR'])/'pipewire-0').exists():
                if audio.poll() is not None or time.monotonic() >= deadline:
                    raise AssertionError('private PipeWire server did not start')
                time.sleep(0.03)
            player = subprocess.Popen([sys.executable, str(ROOT/'scripts/qa/fixtures/mpris.py')],
                                      stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            processes.append(player)
            if not select.select([player.stdout], [], [], 5)[0] or player.stdout.readline().strip() != 'MPRIS_READY':
                raise AssertionError('private MPRIS fixture did not start; install python-dbus-next')
            result = subprocess.run(['quickshell', '-p', str(ROOT/'shell/audio-test.qml')],
                                    cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=20)
            if result.returncode or re.search(r'(HYPRLUME_AUDIO_FAIL|ReferenceError:|TypeError:|Unable to assign|Cannot assign|ERROR:)', result.stdout):
                raise AssertionError(result.stdout)
            for marker in ['HYPRLUME_AUDIO_POLICY_PASS', 'HYPRLUME_AUDIO_NATIVE_PASS']:
                line = next((line for line in result.stdout.splitlines() if marker in line), None)
                if not line:
                    raise AssertionError('missing completion marker: '+result.stdout)
                print(line)
            dump = json.loads(subprocess.check_output(['pw-dump', '-N'], text=True, timeout=4))
            node = next(item for item in dump if item.get('info', {}).get('props', {}).get('node.name') == 'hyprlume-qa-output')
            properties = next(item for item in node['info']['params']['Props'] if 'mute' in item)
            assert properties['mute'], 'PipeWire did not observe mute'
            # PipeWire stores linear amplitude; Quickshell exposes cubic volume.
            assert all(abs(volume - 0.37**3) < 0.002 for volume in properties['channelVolumes']), properties
            metadata = subprocess.check_output(['pw-metadata', '-n', 'default'], text=True, timeout=4)
            assert 'default.configured.audio.sink' in metadata and 'hyprlume-qa-output' in metadata, metadata
            print('AUDIO SERVER PASS: independent PipeWire volume, mute and configured-default observations')
            return 0
        except (AssertionError, subprocess.TimeoutExpired) as error:
            pipewire_log.seek(0)
            print(f'AUDIO CHECK FAILED: {error}\n{pipewire_log.read()}', file=sys.stderr)
            return 1
        finally:
            for process in reversed(processes):
                stop(process)


if __name__ == '__main__':
    raise SystemExit(main())
