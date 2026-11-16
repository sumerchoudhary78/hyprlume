#!/usr/bin/env python3
"""Bake Qt Quick shaders offline; no compiler is needed at UI startup."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qsb', type=Path)
    parser.add_argument('--check', action='store_true', help='verify checked-in source/binary hashes without a compiler')
    args = parser.parse_args()
    directory = ROOT/'shell/assets/shaders'
    sources = sorted(directory.glob('*.frag'))
    manifest = directory/'manifest.json'
    if not sources:
        parser.error('no fragment shader sources found')
    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()
    if args.check:
        if not manifest.is_file(): parser.error('shader manifest missing; run scripts/build-shaders.py')
        entries = json.loads(manifest.read_text())['shaders']
        if set(entries) != {p.name for p in sources}: parser.error('shader manifest does not match sources')
        for source in sources:
            target = source.with_suffix(source.suffix+'.qsb')
            if not target.is_file() or entries[source.name] != {'source_sha256':digest(source),'binary_sha256':digest(target)}:
                parser.error(f'stale shader: {source.name}; run scripts/build-shaders.py')
        print(f'SHADER HASHES PASS: {len(sources)} source/binary pairs')
        return
    candidates = [args.qsb, shutil.which('qsb'), '/usr/lib/qt6/bin/qsb', '/opt/homebrew/opt/qt/bin/qsb']
    qsb = next((Path(p) for p in candidates if p and Path(p).is_file()), None)
    if qsb is None:
        parser.error('qsb not found; install qt6-shadertools or pass --qsb PATH')
    for source in sources:
        target = source.with_suffix(source.suffix + '.qsb')
        fd, name = tempfile.mkstemp(prefix=source.stem+'-', suffix='.qsb', dir=source.parent)
        os.close(fd)
        temp = Path(name)
        try:
            # Qt 6.4 serialization remains readable by the supported Qt 6.9+
            # runtimes. Include SPIR-V, GLSL, HLSL and Metal shader variants.
            subprocess.run([str(qsb), '--qt6', '--qsbversion', '64', '-o', str(temp), str(source)], check=True)
            temp.chmod(0o644)
            temp.replace(target)
            print(f'BAKED {target.relative_to(ROOT)} ({target.stat().st_size} bytes)')
        finally:
            temp.unlink(missing_ok=True)
    manifest.write_text(json.dumps({'qsb_compatibility':'Qt 6.4', 'shaders':{p.name:{'source_sha256':digest(p),'binary_sha256':digest(p.with_suffix(p.suffix+'.qsb'))} for p in sources}},indent=2,sort_keys=True)+'\n')


if __name__ == '__main__':
    main()
