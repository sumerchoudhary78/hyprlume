#!/usr/bin/env python3
"""Exercise the actual Polkit authority and PAM in a disposable CI login."""
import argparse
import json
import os
from pathlib import Path
import pwd
import queue
import secrets
import shutil
import subprocess
import sys
import tempfile
from threading import Thread
import time

ACCOUNT = "hyprlumepolkitqa"
ACTION = "org.hyprlume.qa.authentication"
PAM = Path("/etc/pam.d/hyprlume-polkit-qa")
POLICY = Path("/usr/share/polkit-1/actions") / (ACTION + ".policy")


def provision(binary):
    assert os.environ.get("GITHUB_ACTIONS") == "true" and os.geteuid() == 0
    assert Path("/proc/1/comm").read_text().strip() == "systemd"
    assert not PAM.exists() and not POLICY.exists()
    try:
        pwd.getpwnam(ACCOUNT)
    except KeyError:
        pass
    else:
        raise AssertionError("Refusing to use an existing account")

    subprocess.run(["useradd", "--create-home", "--shell", "/bin/bash", ACCOUNT], check=True)
    try:
        password = secrets.token_urlsafe(24)
        subprocess.run(["chpasswd"], input=f"{ACCOUNT}:{password}\n", text=True, check=True)
        with tempfile.TemporaryDirectory(prefix="hyprlume-polkit-qa-") as directory:
            base = Path(directory)
            shutil.copy2(binary, base / "hyprlume")
            (base / "hyprlume").chmod(0o755)
            shutil.copy2(__file__, base / "check.py")
            (base / "password").write_text(password)
            (base / "password").chmod(0o600)
            account = pwd.getpwnam(ACCOUNT)
            for path in [base, *base.iterdir()]:
                os.chown(path, account.pw_uid, account.pw_gid)
            # Open an actual logind login for this user without changing the
            # distribution's Polkit PAM stack or using an existing session.
            with PAM.open("x") as handle:
                handle.write("account required pam_permit.so\nsession required pam_systemd.so type=wayland class=user\n")
            with POLICY.open("x") as handle:
                handle.write(f"""<?xml version="1.0" encoding="UTF-8"?>
<policyconfig>
  <action id="{ACTION}">
    <description>Hyprlume CI authorization check</description>
    <message>Authentication is required for the Hyprlume CI check</message>
    <defaults>
      <allow_any>auth_self</allow_any>
      <allow_inactive>auth_self</allow_inactive>
      <allow_active>auth_self</allow_active>
    </defaults>
  </action>
</policyconfig>
""")
            return subprocess.run(
                ["systemd-run", "--wait", "--collect", "--pipe", "--quiet",
                 "--unit=hyprlume-polkit-qa", f"--property=User={ACCOUNT}",
                 "--property=PAMName=hyprlume-polkit-qa", "--setenv=GITHUB_ACTIONS=true",
                 "--setenv=XDG_SESSION_TYPE=wayland", "--setenv=WAYLAND_DISPLAY=hyprlume-qa",
                 sys.executable, str(base / "check.py"), "--inside-login", str(base)],
                timeout=100, check=False,
            ).returncode
    finally:
        subprocess.run(["systemctl", "stop", "hyprlume-polkit-qa"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        PAM.unlink(missing_ok=True)
        POLICY.unlink(missing_ok=True)
        subprocess.run(["loginctl", "terminate-user", ACCOUNT], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(["userdel", "--remove", ACCOUNT], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def inside(base):
    assert os.environ.get("GITHUB_ACTIONS") == "true"
    assert pwd.getpwuid(os.geteuid()).pw_name == ACCOUNT
    login = os.environ["XDG_SESSION_ID"]
    status = subprocess.check_output(["loginctl", "show-session", login, "-p", "Type", "-p", "Active", "-p", "Remote"], text=True)
    assert set(status.splitlines()) == {"Type=wayland", "Active=yes", "Remote=no"}, status
    start = Path("/proc/self/stat").read_text().rsplit(")", 1)[1].split()[19]
    process = f"{os.getpid()},{start},{os.geteuid()}"
    password = (base / "password").read_text()
    command = [str(base / "hyprlume"), "polkit-worker"]
    pending = []
    stderr_path = base / "worker.log"
    events = queue.Queue()

    with stderr_path.open("w") as log:
        worker = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log, text=True)

        def read():
            for line in worker.stdout:
                events.put(line)
            events.put(None)

        Thread(target=read, daemon=True).start()

        def send(kind, **fields):
            worker.stdin.write(json.dumps(dict(kind=kind, **fields)) + "\n")
            worker.stdin.flush()

        def wait(kind, timeout=12):
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                try:
                    line = events.get(timeout=max(0.1, deadline - time.monotonic()))
                except queue.Empty:
                    break
                if line is None:
                    raise AssertionError("Polkit worker stopped: " + stderr_path.read_text()[-1500:])
                assert password not in line
                event = json.loads(line)
                if event["kind"] == kind:
                    return event
            raise AssertionError("Timed out waiting for Polkit event " + kind)

        def check():
            proc = subprocess.Popen(["pkcheck", "--action-id", ACTION, "--process", process, "--allow-user-interaction"],
                                    stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
            pending.append(proc)
            return proc

        def start_prompt():
            opened = wait("open")
            assert opened["action"] == ACTION and opened["accounts"] == [ACCOUNT], opened
            send("start", id=opened["id"], account=0)
            prompt = wait("prompt")
            assert prompt["id"] == opened["id"] and prompt["echo"] is False
            return opened["id"], prompt["prompt"]

        try:
            send("available", value=True)
            wait("registered")
            duplicate = subprocess.run(command, input="", stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=8)
            assert duplicate.returncode != 0 and "Register with Polkit" in duplicate.stderr
            assert "registered" not in duplicate.stdout

            first = check()
            request, step = start_prompt()
            overlapping = check()
            assert overlapping.wait(timeout=8) != 0, "A second request was unexpectedly authorized"
            assert first.poll() is None, "A second request replaced the current conversation"
            send("respond", id=request, prompt=step, response="invalid-fixture-credential")
            assert wait("retry")["id"] == request
            send("start", id=request, account=0)
            prompt = wait("prompt")
            assert prompt["prompt"] > step
            send("respond", id=request, prompt=step, response=password)  # Stale step is ignored.
            send("cancel", id=request)
            wait("close")
            assert first.wait(timeout=8) != 0

            second = check()
            request, _ = start_prompt()
            # Lock/disconnect plumbing is verified end to end in the QML test.
            send("available", value=False)
            assert wait("close")["id"] == request
            assert second.wait(timeout=8) != 0

            send("available", value=True)
            third = check()
            request, step = start_prompt()
            send("respond", id=request, prompt=step, response=password)
            wait("close")
            assert third.wait(timeout=8) == 0, third.stderr.read()
            print("POLKIT NATIVE PASS: real authority, duplicate registration, overlapping prompts, PAM failure, cancellation and authorization")
        finally:
            for proc in pending:
                if proc.poll() is None:
                    proc.terminate()
                proc.wait(timeout=5)
            if worker.poll() is None:
                worker.stdin.close()
                try:
                    worker.wait(timeout=6)
                except subprocess.TimeoutExpired:
                    worker.kill()
                    worker.wait()
            assert password not in stderr_path.read_text()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--binary", type=Path)
    mode.add_argument("--inside-login", type=Path)
    args = parser.parse_args()
    if args.binary:
        raise SystemExit(provision(args.binary.resolve()))
    inside(args.inside_login)
