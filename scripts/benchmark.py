#!/usr/bin/env python3
"""Measure a simulated daemon and offscreen native UI without microphone access."""
import argparse
import json
import os
from pathlib import Path
import socket
import statistics
import subprocess
import tempfile
import time


def cpu_ticks(pid):
    fields = Path(f"/proc/{pid}/stat").read_text().rsplit(") ", 1)[1].split()
    return int(fields[11]) + int(fields[12])


def rss(pid):
    for line in Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith("VmRSS:"):
            return round(int(line.split()[1]) / 1024, 2)
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/hyprlume"))
    args = parser.parse_args()
    binary = args.binary.resolve()
    root = Path(__file__).resolve().parents[1]
    with tempfile.TemporaryDirectory(prefix="hyprlume-bench-") as temp:
        path = Path(temp) / "private/control.sock"
        daemon = subprocess.Popen([str(binary), "--socket", str(path), "daemon", "--demo"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        ui = None
        try:
            deadline = time.monotonic() + 5
            while not path.exists():
                if daemon.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError("daemon did not start")
                time.sleep(0.01)
            env = dict(os.environ, QT_QPA_PLATFORM="offscreen", QT_QUICK_BACKEND="software", HYPRLUME_REDUCED_EFFECTS="1", LANG="C.UTF-8")
            ui = subprocess.Popen([str(binary), "--socket", str(path), "shell", "--path", str(root / "shell"), "--preview"], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            time.sleep(1)
            if ui.poll() is not None:
                raise RuntimeError("UI did not remain running")
            with socket.socket(socket.AF_UNIX) as client:
                client.settimeout(2)
                client.connect(str(path))
                file = client.makefile("rb")
                samples = []
                for index in range(550):
                    start = time.perf_counter_ns()
                    client.sendall((json.dumps({"id": index, "op": "status"}) + "\n").encode())
                    response = json.loads(file.readline(8192))
                    assert response["id"] == index and "result" in response
                    if index >= 50:
                        samples.append((time.perf_counter_ns() - start) / 1_000_000)
            before = [cpu_ticks(daemon.pid), cpu_ticks(ui.pid)]
            start = time.monotonic()
            time.sleep(3)
            elapsed = time.monotonic() - start
            after = [cpu_ticks(daemon.pid), cpu_ticks(ui.pid)]
            hz = os.sysconf("SC_CLK_TCK")
            samples.sort()
            print(json.dumps({
                "environment": "Linux; demo daemon; reduced-effects dashboard; Qt offscreen software renderer; live host metrics; voice disabled",
                "samples": len(samples),
                "status_rtt_ms_p50": round(statistics.median(samples), 4),
                "status_rtt_ms_p95": round(samples[int(len(samples) * 0.95)], 4),
                "status_rtt_ms_p99": round(samples[int(len(samples) * 0.99)], 4),
                "daemon_rss_mib": rss(daemon.pid),
                "shell_rss_mib": rss(ui.pid),
                "idle_seconds": round(elapsed, 2),
                "daemon_idle_cpu_percent_one_core": round((after[0] - before[0]) / hz / elapsed * 100, 3),
                "shell_idle_cpu_percent_one_core": round((after[1] - before[1]) / hz / elapsed * 100, 3),
            }, indent=2))
        finally:
            for process in [ui, daemon]:
                if process is not None:
                    process.terminate()
                    try:
                        process.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()


if __name__ == "__main__":
    main()
