#!/usr/bin/env python3
"""Verify the native XDG index and GIO launch on an isolated display and bus."""
import argparse
import os
from pathlib import Path
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
        with tempfile.TemporaryDirectory(prefix='hyprlume-launcher-qa-') as temporary:
            base = Path(temporary)
            apps = base/'data/applications'; apps.mkdir(parents=True)
            fixture = base/'marker.py'
            fixture.write_text('import os, pathlib, sys\npathlib.Path(os.environ["HYPRLUME_QA_MARKER"]).write_text("|".join(sys.argv[1:]))\n')
            for name, extra in [('hyprlume-qa',''),('hidden','Hidden=true'),('nodisplay','NoDisplay=true')]:
                title = {'hidden':'Hidden QA','nodisplay':'NoDisplay QA'}.get(name, 'Quasar QA')
                (apps/(name+'.desktop')).write_text(f'[Desktop Entry]\nType=Application\nName={title}\nKeywords=quasar;orbit;\nExec="{sys.executable}" "{fixture}" %c %%\n{extra}\n')
            env = os.environ.copy()
            for name in ['WAYLAND_DISPLAY','HYPRLAND_INSTANCE_SIGNATURE','DISPLAY','DBUS_SESSION_BUS_ADDRESS']:
                env.pop(name,None)
            env.update(XDG_RUNTIME_DIR=str(base),XDG_DATA_HOME=str(base/'data'),XDG_DATA_DIRS=str(base/'empty'),XDG_CONFIG_HOME=str(base),
                       QT_QPA_PLATFORM='xcb',LIBGL_ALWAYS_SOFTWARE='1',HYPRLUME_QA_MARKER=str(base/'marker'))
            return subprocess.call(['xvfb-run','-a','dbus-run-session','--',sys.executable,__file__,'--inside-display'],env=env,cwd=ROOT)
    result = subprocess.run(['quickshell','-p',str(ROOT/'shell/launcher-test.qml')],capture_output=True,text=True,timeout=15)
    assert result.returncode == 0 and 'HYPRLUME_LAUNCHER_NATIVE_PASS' in result.stdout+result.stderr, result.stdout+result.stderr
    subprocess.run(['gtk-launch','--','hyprlume-qa.desktop'],check=True,timeout=5)
    marker = Path(os.environ['HYPRLUME_QA_MARKER'])
    for _ in range(50):
        if marker.exists(): break
        time.sleep(.05)
    assert marker.read_text() == 'Quasar QA|%', marker.read_text()
    print('LAUNCHER NATIVE PASS: XDG visibility, keyword search, stale selection and GIO Exec field codes')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
