#!/usr/bin/env python3
"""Exercise wake → ASR → action preview with synthetic PCM, without a microphone."""
import argparse
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time
import wave


def pcm16k(path):
    with wave.open(str(path)) as wav:
        assert wav.getnchannels() == 1 and wav.getsampwidth() == 2
        rate = wav.getframerate()
        data = wav.readframes(wav.getnframes())
    samples = struct.unpack("<" + "h" * (len(data) // 2), data)
    output = []
    for index in range(int(len(samples) * 16000 / rate)):
        position = index * rate / 16000
        low = int(position)
        high = min(low + 1, len(samples) - 1)
        output.append(round(samples[low] * (1 - position + low) + samples[high] * (position - low)))
    return struct.pack("<" + "h" * len(output), *output)


def request(path, operation):
    with socket.socket(socket.AF_UNIX) as client:
        client.settimeout(3)
        client.connect(str(path))
        client.sendall((json.dumps(dict(id=1, **operation)) + "\n").encode())
        response = json.loads(client.makefile("rb").readline(8192))
        if "error" in response:
            raise RuntimeError(response["error"])
        return response["result"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/hyprlume"))
    args = parser.parse_args()
    binary = args.binary.resolve()
    with tempfile.TemporaryDirectory(prefix="hyprlume-voice-") as temp:
        root = Path(temp)
        wake, command = root / "wake.wav", root / "command.wav"
        for text, output in [("Hey Pilot.", wake), ("Go to work space two.", command)]:
            subprocess.run([str(binary), "speak", text, "--output", str(output)], check=True)
        pcm = root / "speech.pcm"
        pcm.write_bytes(bytes(16000) + pcm16k(wake) + bytes(32000) + pcm16k(command) + bytes(96000))
        tools = root / "tools"
        tools.mkdir()
        recorder = tools / "pw-record"
        recorder.write_text("#!/usr/bin/env python3\nimport os,sys,time\nwith open(os.environ['HYPRLUME_TEST_PCM'],'rb') as f:\n while data:=f.read(640):\n  sys.stdout.buffer.write(data);sys.stdout.buffer.flush();time.sleep(.02)\n time.sleep(5)\n")
        recorder.chmod(0o755)
        config = root / "config.toml"
        config.write_text("[voice]\nexecute_commands=false\nspeak_results=false\n")
        path = root / "private/control.sock"
        env = dict(os.environ, PATH=str(tools) + os.pathsep + os.environ["PATH"], HYPRLUME_TEST_PCM=str(pcm))
        daemon = subprocess.Popen([str(binary), "--config", str(config), "--socket", str(path), "daemon"], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            deadline = time.monotonic() + 5
            while not path.exists():
                if daemon.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError("daemon did not start")
                time.sleep(.01)
            request(path, {"op": "voice", "mode": "wake"})
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                state = request(path, {"op": "status"})
                if state["phase"] == "error":
                    raise RuntimeError(json.dumps(state))
                if state["phase"] == "review":
                    assert state["actions"] == ["Switch to workspace 2"], state
                    assert state["confirmation"] is not None and not state["receipts"], state
                    request(path, {"op": "voice", "mode": "off"})
                    assert request(path, {"op": "status"})["voice"] == "off"
                    print(json.dumps({"passed": True, "transcript": state["transcript"], "actions": state["actions"], "microphone": "never opened; injected PCM capture process", "desktop": "preview only; no mutation"}, indent=2))
                    return
                time.sleep(.05)
            raise RuntimeError("voice preview did not arrive: " + json.dumps(state))
        finally:
            daemon.terminate()
            try:
                daemon.wait(timeout=3)
            except subprocess.TimeoutExpired:
                daemon.kill()
                daemon.wait()


if __name__ == "__main__":
    main()
