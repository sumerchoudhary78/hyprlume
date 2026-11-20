"""Event and real-input driver for the isolated Quickshell fixture."""
import json
import queue
import re
import subprocess
import threading
import time


class UiDriver:
    def __init__(self, proc, dpr=1):
        self.proc = proc
        self.dpr = dpr
        self.events = queue.Queue()
        self.logs = []
        self.controls = {}
        self.thread = threading.Thread(target=self._read_logs, daemon=True)
        self.thread.start()

    def _read_logs(self):
        for line in self.proc.stdout:
            self.logs.append(line)
            match = re.search(r'HYPRLUME_QA (\{.*\})', line)
            if match:
                self.events.put(json.loads(match.group(1)))

    def wait_event(self, predicate, timeout=6):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            if self.proc.poll() is not None:
                raise AssertionError('QML exited before the interaction completed')
            try:
                event = self.events.get(timeout=0.1)
            except queue.Empty:
                continue
            if event['kind'] == 'controls':
                self.controls.clear()
                self.controls.update(event['controls'])
            if predicate(event):
                return event
        raise AssertionError('timed out waiting for a UI event')

    def xdo(self, *parts):
        return subprocess.check_output(['xdotool', *map(str, parts)], text=True).strip()

    def wait_controls(self, name, present=True):
        self.wait_event(
            lambda event: event['kind'] == 'controls'
            and ((name in event['controls']) == present)
        )

    def wait_stable_controls(self, *names):
        previous = None

        def stable(event):
            nonlocal previous
            if event['kind'] != 'controls':
                return False
            current = tuple(event['controls'].get(name) for name in names)
            ready = current == previous and all(control and control['enabled'] for control in current)
            previous = current
            return ready

        self.wait_event(stable)

    def click(self, name):
        if name not in self.controls:
            raise AssertionError(f'control missing: {name}; found {list(self.controls)}')
        position = self.controls[name]
        if not position['enabled']:
            raise AssertionError(f'control is disabled: {name}')
        self.click_at(position)

    def click_at(self, position, button=1, repeat=1):
        x = round(position['x'] * self.dpr)
        y = round(position['y'] * self.dpr)
        self.xdo('mousemove', '--sync', x, y, 'click', '--repeat', repeat, button)

    def request(self, op):
        event = self.wait_event(
            lambda event: event['kind'] == 'request' and event['operation']['op'] == op
        )
        return event['operation']

    def focus_window(self):
        window = self.xdo('search', '--onlyvisible', '--name', '^Hyprlume Design Preview$').splitlines()[0]
        self.xdo('windowfocus', '--sync', window)

    def stop(self):
        if self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
        self.thread.join(timeout=2)
