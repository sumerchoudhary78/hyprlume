#!/usr/bin/env python3
"""CI-only real Wayland/PAM qualification in a disposable Linux user session.

Provisioning is refused outside GitHub Actions. This script never targets an
existing user, session, clipboard, compositor, display or PAM configuration.
"""
import argparse
import json
import os
from pathlib import Path
import pwd
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time

ROOT=Path(__file__).resolve().parents[2]
UNIT='hyprlume-lock.service'

def run(arguments,**options):
    return subprocess.run(arguments,check=True,text=True,**options)

def wait(predicate,label,seconds=15):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        result=predicate()
        if result:return result
        time.sleep(.08)
    raise AssertionError('timed out: '+label)

def provision(binary):
    assert os.environ.get('GITHUB_ACTIONS')=='true' and os.geteuid()==0, 'Provisioning is only supported in an ephemeral GitHub Actions VM'
    assert Path('/proc/1/comm').read_text().strip()=='systemd','A real systemd VM is required'
    user='hyprlumeqa'
    try:pwd.getpwnam(user)
    except KeyError:pass
    else:raise AssertionError('Refusing to use an existing account')
    run(['useradd','--create-home','--shell','/bin/bash',user])
    account=pwd.getpwnam(user);password=secrets.token_urlsafe(20)
    run(['chpasswd'],input=user+':'+password+'\n')
    base=Path(tempfile.mkdtemp(prefix='hyprlume-wayland-qa-'))
    try:
        (base/'bin').mkdir();shutil.copy2(binary,base/'bin/hyprlume')
        assets=base/'share/hyprlume/shell/assets/backgrounds';assets.mkdir(parents=True)
        shutil.copy2(ROOT/'shell/assets/backgrounds/lock.svg',assets/'lock.svg')
        shutil.copy2(__file__,base/'check.py');(base/'password').write_text(password);(base/'password').chmod(0o600)
        for path in [base,*base.rglob('*')]:os.chown(path,account.pw_uid,account.pw_gid)
        run(['loginctl','enable-linger',user]);run(['systemctl','start',f'user@{account.pw_uid}.service'])
        result=subprocess.run(['runuser','-u',user,'--',sys.executable,str(base/'check.py'),'--inside-user','--base',str(base)],text=True)
        output=ROOT/'output/design/wayland';output.mkdir(parents=True,exist_ok=True)
        for name in ['sway.log','daemon.log','lock-journal.log','report.json','private-screenshot.png']:
            if (base/name).exists():
                shutil.copyfile(base/name,output/name);(output/name).chmod(0o644)
        run(['rsvg-convert','-w','1920','-h','1080','-o',str(output/'lock-background.png'),str(assets/'lock.svg')])
        return result.returncode
    finally:
        subprocess.run(['loginctl','terminate-user',user],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        subprocess.run(['loginctl','disable-linger',user],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        subprocess.run(['userdel','--remove',user],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        shutil.rmtree(base,ignore_errors=True)

def inside(base):
    assert os.environ.get('GITHUB_ACTIONS')=='true' and os.geteuid()!=0
    assert pwd.getpwuid(os.geteuid()).pw_name=='hyprlumeqa'
    runtime=Path('/run/user')/str(os.geteuid());assert runtime.is_dir()
    env=os.environ.copy()
    for key in ['DISPLAY','WAYLAND_DISPLAY','HYPRLAND_INSTANCE_SIGNATURE','SWAYSOCK']:env.pop(key,None)
    env.update(XDG_RUNTIME_DIR=str(runtime),DBUS_SESSION_BUS_ADDRESS='unix:path='+str(runtime/'bus'),XDG_SESSION_TYPE='wayland',
               XDG_CONFIG_HOME=str(base/'config'),XDG_DATA_HOME=str(base/'data'),XDG_PICTURES_DIR=str(base/'pictures'),XDG_VIDEOS_DIR=str(base/'videos'),
               WLR_BACKENDS='headless',WLR_RENDERER='pixman',WLR_RENDERER_ALLOW_SOFTWARE='1',WLR_LIBINPUT_NO_DEVICES='1')
    (base/'sway.conf').write_text('output * mode 1280x720\noutput * bg #102338 solid_color\nseat seat0 fallback true\n')
    sway_log=(base/'sway.log').open('w');daemon_log=(base/'daemon.log').open('a')
    sway=subprocess.Popen(['sway','--unsupported-gpu','--config',str(base/'sway.conf')],env=env,stdout=sway_log,stderr=subprocess.STDOUT)
    daemon=None
    binary=base/'bin/hyprlume';control=runtime/'hyprlume-qa/control.sock'
    def systemctl(*arguments):return subprocess.check_output(['systemctl','--user',*arguments],env=env,text=True).strip()
    def call(op,**fields):
        with socket.socket(socket.AF_UNIX) as client:
            client.settimeout(5);client.connect(str(control));client.sendall(json.dumps(dict(id=1,op=op,**fields)).encode()+b'\n')
            return json.loads(client.makefile('rb').readline())
    def state(topic):
        response=call(topic+'_status');assert 'error' not in response,response
        return response['result']
    def start_daemon():
        process=subprocess.Popen(['setpriv','--no-new-privs',str(binary),'--socket',str(control),'daemon'],env=env,stdout=daemon_log,stderr=subprocess.STDOUT)
        wait(lambda:control.exists() or process.poll() is not None,'daemon socket')
        assert process.poll() is None,'daemon exited'
        return process
    def type_text(text):
        run(['wtype','-'],env=env,input=text,stdout=subprocess.DEVNULL)
        run(['wtype','-k','Return'],env=env,stdout=subprocess.DEVNULL)
    def worker_pid():
        value=systemctl('show',UNIT,'--property=MainPID','--value')
        return int(value) if value.isdigit() else 0
    def client_pid():
        parent=worker_pid()
        if not parent:return 0
        result=subprocess.run(['pgrep','-P',str(parent),'-x','swaylock'],env=env,capture_output=True,text=True)
        return int(result.stdout.strip().splitlines()[0]) if result.returncode==0 else 0
    try:
        def display_socket():
            return next((path for path in runtime.glob('wayland-*') if path.is_socket()),None)
        display=wait(display_socket,'private compositor')
        assert sway.poll() is None
        env['WAYLAND_DISPLAY']=display.name
        ipc=wait(lambda:next(runtime.glob('sway-ipc.*.sock'),None),'private Sway IPC')
        env['SWAYSOCK']=str(ipc)
        daemon=start_daemon()
        assert 'NoNewPrivs:\t1' in Path(f'/proc/{daemon.pid}/status').read_text()
        assert state('lock')['available'],state('lock')
        call('clipboard',request={'action':'enable','enabled':True})
        time.sleep(.3)
        run(['wl-copy','--type','text/plain'],env=env,input='private Wayland fixture',stdout=subprocess.DEVNULL)
        wait(lambda:state('clipboard')['items'],'native clipboard delivery')
        call('capture',request={'action':'screenshot','region':False})
        capture=wait(lambda:value if (value:=state('capture'))['output'].endswith('.png') and not value['busy'] else None,'native screenshot')
        image=Path(capture['output']);assert image.read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
        assert image.stat().st_mode&0o777==0o600
        shutil.copy2(image,base/'private-screenshot.png')
        outputs=json.loads(subprocess.check_output(['swaymsg','-t','get_outputs','-r'],env=env,text=True))
        call('capture',request={'action':'record','region':False,'output':outputs[0]['name']})
        wait(lambda:state('capture')['recording'],'native screen recorder')
        run(['swaymsg','output * bg #19394c solid_color'],env=env,stdout=subprocess.DEVNULL)
        time.sleep(.6)
        accepted=call('lock');assert accepted.get('result',{}).get('accepted'),accepted
        locked=wait(lambda:value if (value:=state('lock'))['phase']=='locked' else None,'secure lock readiness',35)
        assert locked['active']
        run([str(binary),'--socket',str(control),'lock','--wait'],env=env,stdout=subprocess.DEVNULL,timeout=10)
        assert not state('clipboard')['enabled'] and not state('clipboard')['items']
        capture=state('capture');assert not capture['busy'] and capture['output'].endswith('.mkv'),capture
        assert not capture['error'],capture
        streams=json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',capture['output']],text=True))['streams']
        assert streams and all(stream['codec_type']!='audio' for stream in streams)
        assert 'error' in call('command',text='workspace 2',dry_run=False)
        assert 'error' in call('voice',mode='wake')
        worker=worker_pid();assert worker
        assert 'NoNewPrivs:\t0' in Path(f'/proc/{worker}/status').read_text()
        client=wait(client_pid,'lock client PID')
        os.kill(client,signal.SIGUSR1);time.sleep(.5)
        assert state('lock')['active'] and client_pid()==client,'SIGUSR1 bypassed authentication'
        type_text('incorrect-fixture-password');time.sleep(4)
        assert state('lock')['active'] and worker_pid()==worker,'incorrect password ended locking'
        # Restart only our private daemon: the independent lock unit must survive.
        daemon.terminate();daemon.wait(timeout=15);daemon=start_daemon()
        wait(lambda:state('lock')['phase']=='locked','lock observation after daemon restart')
        assert worker_pid()==worker
        # Kill only the known lock child; the compositor stays fail-closed and
        # the dedicated unit restores the authentication surface.
        os.kill(client,signal.SIGKILL)
        wait(lambda:worker_pid() not in [0,worker],'lock unit restart')
        wait(lambda:state('lock')['phase']=='locked' and client_pid() not in [0,client],'lock recovery',20)
        assert (runtime/'hyprlume-lock/episode').exists()
        type_text((base/'password').read_text())
        wait(lambda:not state('lock')['active'],'PAM authentication',15)
        assert not (runtime/'hyprlume-lock/episode').exists()
        assert call('status')['result']['voice']=='off'
        assert not state('clipboard')['enabled']
        report={'clipboard':'real data-control delivery','screenshot':'valid native PNG','recording':'real finalized silent video',
                'locking':'ext-session-lock readiness and real PAM','wrong_password':'rejected','SIGUSR1':'blocked',
                'daemon_restart':'lock survived','client_crash':'fail-closed recovery','PAM_password':'unlocked','sandbox':'daemon NNP=1, lock worker NNP=0'}
        (base/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        print('WAYLAND SERVICES PASS: '+json.dumps(report))
        return 0
    finally:
        with (base/'lock-journal.log').open('w') as journal:
            subprocess.run(['journalctl','--user','-u',UNIT,'--no-pager','-n','100'],env=env,stdout=journal,stderr=subprocess.STDOUT)
        if daemon is not None:
            daemon.terminate()
            try:daemon.wait(timeout=15)
            except subprocess.TimeoutExpired:daemon.kill();daemon.wait()
        subprocess.run(['systemctl','--user','stop',UNIT],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        sway.terminate()
        try:sway.wait(timeout=5)
        except subprocess.TimeoutExpired:sway.kill();sway.wait()
        sway_log.close();daemon_log.close()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=ROOT/'target/debug/hyprlume')
    parser.add_argument('--ci-provision',action='store_true');parser.add_argument('--inside-user',action='store_true');parser.add_argument('--base',type=Path)
    args=parser.parse_args()
    if args.inside_user:return inside(args.base)
    assert args.ci_provision,'This check provisions only a disposable CI account; use --ci-provision in GitHub Actions'
    return provision(args.binary.resolve())
if __name__=='__main__':raise SystemExit(main())
