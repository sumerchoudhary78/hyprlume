#!/usr/bin/env python3
"""Exercise settings through the real daemon and a private custom config path."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import tomllib

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,default=Path('target/debug/hyprlume'));args=parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='hyprlume-settings-qa-') as temporary:
        base=Path(temporary);runtime=base/'runtime';runtime.mkdir(mode=0o700)
        path=base/'custom.toml';path.write_text('[apps]\nterminal = ["kitty"]\n')
        control=runtime/'control.sock';env=os.environ.copy()
        env.update(XDG_RUNTIME_DIR=str(runtime),XDG_CONFIG_HOME=str(base/'unused-default-config'),XDG_DATA_HOME=str(base/'data'))
        env.pop('HYPRLAND_INSTANCE_SIGNATURE',None);env.pop('WAYLAND_DISPLAY',None)
        def start():
            daemon=subprocess.Popen([str(args.binary.resolve()),'--config',str(path),'--socket',str(control),'daemon'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            deadline=time.monotonic()+5
            while not control.exists() and daemon.poll() is None and time.monotonic()<deadline:time.sleep(.03)
            assert daemon.poll() is None and control.exists(),'settings daemon did not start'
            return daemon
        def call(op,**fields):
            with socket.socket(socket.AF_UNIX) as client:
                client.settimeout(4);client.connect(str(control));client.sendall(json.dumps(dict(id=1,op=op,**fields)).encode()+b'\n')
                return json.loads(client.makefile('rb').readline())
        daemon=start()
        try:
            state=call('settings_status')['result'];assert state['path']==str(path)
            values=state['values'];values['preferences']['appearance']['accent']='violet';values['voice']['threads']=3
            values['preferences']['services']['notifications_dnd']=True
            updated=call('settings',request=dict(action='save',revision=state['revision'],values=values))['result']
            assert updated['restart_required'];assert path.stat().st_mode&0o777==0o600
            saved=tomllib.loads(path.read_text());assert saved['apps']['terminal']==['kitty']
            assert saved['voice']['threads']==3
            assert 'error' in call('settings',request=dict(action='save',revision=state['revision'],values=values))
            before=path.read_bytes();values['voice']['threads']=99
            assert 'error' in call('settings',request=dict(action='save',revision=updated['revision'],values=values))
            assert path.read_bytes()==before
            values['voice']['threads']=3;path.write_bytes(before+b'\n# external editor\n')
            assert 'error' in call('settings',request=dict(action='save',revision=updated['revision'],values=values))
            assert call('settings',request={'action':'reload'})['result']['revision']>updated['revision']
            daemon.terminate();daemon.wait(timeout=12);daemon=start()
            restored=call('settings_status')['result'];assert not restored['restart_required']
            assert restored['values']['preferences']['appearance']['accent']=='violet'
            assert call('notifications_status')['result']['dnd']
            assert call('status')['result']['voice']=='off'
            assert not (base/'unused-default-config/hyprlume/config.toml').exists()
            print('SETTINGS PASS: custom path, atomic/private save, validation, revision/external-edit conflicts, restart loading, DND default and microphone off')
        finally:
            daemon.terminate();daemon.wait(timeout=12)
    return 0
if __name__=='__main__':raise SystemExit(main())
