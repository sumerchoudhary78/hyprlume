#!/usr/bin/env python3
"""Validate the enabled user-service graph without starting any units."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/hyprlume"))
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(f"Build the executable first: {binary}")
    source = Path(__file__).resolve().parents[1] / "packaging/systemd"
    with tempfile.TemporaryDirectory(prefix="hyprlume-unit-check-") as temporary:
        root = Path(temporary)
        runtime = root / "runtime"
        runtime.mkdir(mode=0o700)
        target = root / "graphical-session.target"
        target.write_text("[Unit]\nDescription=Isolated graphical-session validation\nRequires=basic.target\n")
        units = []
        for name in ["hyprlume-daemon.service", "hyprlume-shell.service", "hyprlume-polkit.service",
                     "hyprlume-portal-settings.service", "hyprlume-session.target"]:
            path = root / name
            path.write_text((source / name).read_text().replace("/usr/bin/hyprlume", str(binary)))
            units.append(str(path))
        env = dict(os.environ, SYSTEMD_UNIT_PATH=str(root) + ":", XDG_RUNTIME_DIR=str(runtime))
        subprocess.run(["systemd-analyze", "--user", "verify", "--man=no", str(target), *units], env=env, check=True)
    print("Enabled user-service graph verified; no units were started")


if __name__ == "__main__":
    main()
