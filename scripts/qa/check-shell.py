#!/usr/bin/env python3
"""Smoke the actual composition root, offline, on a private X display and bus."""
import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
ROOT=Path(__file__).resolve().parents[2]
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--inside-display',action='store_true');args=parser.parse_args()
    if not args.inside_display:
        with tempfile.TemporaryDirectory(prefix='hyprlume-shell-qa-') as temporary:
            env=os.environ.copy()
            for key in ['DISPLAY','WAYLAND_DISPLAY','HYPRLAND_INSTANCE_SIGNATURE','DBUS_SESSION_BUS_ADDRESS']:env.pop(key,None)
            env.update(XDG_RUNTIME_DIR=temporary,XDG_CONFIG_HOME=temporary,XDG_STATE_HOME=temporary,HYPRLUME_SOCKET=str(Path(temporary)/'offline.sock'),
                       QT_QPA_PLATFORM='xcb',LIBGL_ALWAYS_SOFTWARE='1',HYPRLUME_PREVIEW='1',HYPRLUME_QA='1')
            return subprocess.call(['xvfb-run','-a','dbus-run-session','--',sys.executable,__file__,'--inside-display'],env=env,cwd=ROOT)
    proc=subprocess.Popen(['quickshell','-p',str(ROOT/'shell/shell.qml')],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
    try:
        try:output=proc.communicate(timeout=4)[0]
        except subprocess.TimeoutExpired:
            proc.terminate();output=proc.communicate(timeout=5)[0]
        assert 'HYPRLUME_LIVE_PREVIEW_READY' in output,output
        assert not re.search(r'(ReferenceError:|TypeError:|Unable to assign|Cannot assign|Binding loop|Failed to load|Failed to deserialize|ERROR:)',output),output
        print('LIVE SHELL PASS: actual entry, native services, offline backend and preview composition load without binding errors')
        return 0
    finally:
        if proc.poll() is None:proc.kill();proc.wait()
if __name__=='__main__':raise SystemExit(main())
