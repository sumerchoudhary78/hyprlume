#!/usr/bin/env python3
"""Read CPU/RSS for the current user's running Hyprlume services on Linux.

No desktop, voice, or service changes are made. CPU comes from each service's
cgroup, including its short-lived sampler children. Percentages use one core.
"""
import argparse
import json
from pathlib import Path
import subprocess
import time


def service_group(name):
    subprocess.run(['systemctl', '--user', 'is-active', '--quiet', name], check=True)
    group = subprocess.check_output(['systemctl', '--user', 'show', name,
                                     '--property=ControlGroup', '--value'], text=True).strip()
    if not group:
        raise RuntimeError(f'{name} has no cgroup')
    return Path('/sys/fs/cgroup')/group.lstrip('/')


def usage(group):
    counters = dict(line.split() for line in (group/'cpu.stat').read_text().splitlines())
    return int(counters['usage_usec'])


def rss(group):
    pids = {pid for file in group.rglob('cgroup.procs') for pid in file.read_text().split()}
    total = 0
    for pid in pids:
        try:
            lines = Path(f'/proc/{pid}/status').read_text().splitlines()
        except (FileNotFoundError, ProcessLookupError):
            continue
        total += sum(int(line.split()[1]) for line in lines if line.startswith('VmRSS:'))
    return round(total/1024, 2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seconds', type=float, default=20)
    parser.add_argument('--label', default='current session; caller records visibility/effects/voice')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 60:
        parser.error('--seconds must be between 1 and 60')
    groups = {name: service_group(f'hyprlume-{name}.service') for name in ['daemon', 'shell']}
    before = {name: usage(group) for name, group in groups.items()}
    start = time.monotonic()
    time.sleep(args.seconds)
    elapsed = time.monotonic()-start
    result = {
        'environment': args.label, 'duration_seconds': round(elapsed, 2),
        'cpu_accounting': 'systemd service cgroup usage_usec; includes sampler children',
        'process_tree_cpu_percent_one_core': {
            name: round((usage(group)-before[name])/1_000_000/elapsed*100, 3)
            for name, group in groups.items()
        },
        'rss_mib': {name: rss(group) for name, group in groups.items()},
    }
    rendered = json.dumps(result, indent=2)+'\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end='')


if __name__ == '__main__':
    main()
