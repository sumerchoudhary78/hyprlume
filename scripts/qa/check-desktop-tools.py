#!/usr/bin/env python3
"""Run the Rust daemon against private clipboard/capture executables, never the desktop."""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import time

FIXTURE = r'''#!/usr/bin/env python3
import json, os, pathlib, signal, subprocess, sys, time
name=pathlib.Path(sys.argv[0]).name
base=pathlib.Path(os.environ['HYPRLUME_TOOL_QA'])
if name=='wl-paste':
    # Deliver two normal offers and a sensitive offer through the real ingest CLI.
    index=sys.argv.index('--watch')+1
    delivery=sys.argv[index:]
    (base/'delivery.json').write_text(json.dumps(delivery))
    for text,sensitive in [('first private fixture',False),('second private fixture',False),('never store this fixture',True)]:
        env=os.environ.copy();env['CLIPBOARD_STATE']='sensitive' if sensitive else 'data'
        subprocess.run(delivery,input=text.encode(),env=env,check=True)
    while True:time.sleep(1)
elif name=='wl-copy':
    (base/'copied').write_bytes(sys.stdin.buffer.read())
elif name=='slurp':
    print('0,0 640x480')
elif name=='grim':
    sys.stdout.buffer.write(b'PNG fixture only')
elif name=='wf-recorder':
    output=pathlib.Path(sys.argv[sys.argv.index('--file')+1])
    output.write_bytes(b'MKV fixture only')
    (base/'recorder-ready').write_text(str(os.getpid()))
    def stop(signum,frame):
        with output.open('ab') as writer:writer.write(b' finalized')
        (base/'recorder-stopped').write_text(str(signum));sys.exit(0)
    signal.signal(signal.SIGINT,stop)
    while True:time.sleep(.1)
'''

def wait(predicate):
    end=time.monotonic()+6
    while time.monotonic()<end:
        value=predicate()
        if value:return value
        time.sleep(.04)
    raise AssertionError('desktop tool fixture timed out')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/hyprlume'))
    args=parser.parse_args();binary=args.binary.resolve()
    with tempfile.TemporaryDirectory(prefix='hyprlume-tools-qa-') as temporary:
        base=Path(temporary);tools=base/'bin';tools.mkdir()
        for name in ['wl-paste','wl-copy','grim','slurp','wf-recorder']:
            executable=tools/name;executable.write_text(FIXTURE);executable.chmod(0o700)
        runtime=base/'runtime';runtime.mkdir(mode=0o700)
        control=runtime/'control.sock'
        env=os.environ.copy()
        env.update(PATH=str(tools)+os.pathsep+env['PATH'],XDG_RUNTIME_DIR=str(runtime),XDG_CONFIG_HOME=str(base/'config'),XDG_DATA_HOME=str(base/'data'),
                   XDG_PICTURES_DIR=str(base/'pictures'),XDG_VIDEOS_DIR=str(base/'videos'),WAYLAND_DISPLAY='isolated-fixture-only',HYPRLUME_TOOL_QA=str(base))
        env.pop('HYPRLAND_INSTANCE_SIGNATURE',None)
        daemon=subprocess.Popen([str(binary),'--socket',str(control),'daemon'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
        try:
            wait(lambda:control.exists() or daemon.poll() is not None)
            assert daemon.poll() is None,'daemon failed to start'
            def call(op,**fields):
                with socket.socket(socket.AF_UNIX) as client:
                    client.settimeout(4);client.connect(str(control));client.sendall(json.dumps(dict(id=1,op=op,**fields)).encode()+b'\n')
                    response=json.loads(client.makefile('rb').readline())
                    if 'error' in response:raise AssertionError(response['error'])
                    return response['result']
            assert call('clipboard_status')['enabled'] is False
            call('clipboard',request={'action':'enable','enabled':True})
            history=wait(lambda: (state if len((state:=call('clipboard_status'))['items'])==2 else None))
            assert all('never store' not in entry['preview'] for entry in history['items'])
            assert all('text' not in entry for entry in history['items'])
            call('clipboard',request={'action':'copy','id':history['items'][0]['id']})
            assert (base/'copied').read_text()=='second private fixture'
            call('clipboard',request={'action':'enable','enabled':False})
            delivery=json.loads((base/'delivery.json').read_text());generation=int(delivery[-1])
            with socket.socket(socket.AF_UNIX) as client:
                client.settimeout(3);client.connect(str(control));client.sendall(json.dumps({'id':3,'op':'clipboard_ingest','generation':generation,'text':'late fixture'}).encode()+b'\n')
                assert 'error' in json.loads(client.makefile('rb').readline())
            call('clipboard',request={'action':'clear'})
            assert not call('clipboard_status')['items']
            call('capture',request={'action':'screenshot','region':True})
            capture=wait(lambda:state if (state:=call('capture_status'))['output'].endswith('.png') and not state['busy'] else None)
            screenshot=Path(capture['output']);assert screenshot.read_bytes()==b'PNG fixture only'
            assert screenshot.stat().st_mode&0o777==0o600
            assert screenshot.parent.stat().st_mode&0o777==0o700
            call('capture',request={'action':'record','region':False})
            wait(lambda:(base/'recorder-ready').exists())
            assert call('capture_status')['recording']
            call('capture',request={'action':'stop'})
            capture=wait(lambda:state if not (state:=call('capture_status'))['busy'] and state['output'].endswith('.mkv') else None)
            assert not capture['error'],capture
            assert Path(capture['output']).read_bytes().endswith(b' finalized')
            assert (base/'recorder-stopped').read_text()==str(signal.SIGINT)
            print('DESKTOP TOOLS PASS: real socket/watch delivery, sensitive rejection, stale generation, copy stdin, private screenshot and finalized owned recording')
        finally:
            daemon.terminate()
            try:daemon.wait(timeout=12)
            except subprocess.TimeoutExpired:daemon.kill();daemon.wait()
            stderr=daemon.stderr.read().decode(errors='replace')
            assert 'first private fixture' not in stderr and 'second private fixture' not in stderr and 'never store' not in stderr
    return 0

if __name__=='__main__':raise SystemExit(main())
