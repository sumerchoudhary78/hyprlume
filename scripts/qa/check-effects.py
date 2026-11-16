#!/usr/bin/env python3
"""Check the actual rendered liquid areas and the shared animation lifecycle.

Requires the native renderer's tools plus Pillow. Uses an isolated display and
sample data; never opens the microphone or controls the running desktop.
"""
import json
import math
from pathlib import Path
import subprocess
import sys

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]


def main():
    output = ROOT/'output/design/liquid-materials.png'
    rendered = subprocess.run([
        sys.executable, str(ROOT/'scripts/qa/render-desktop.py'), '--effects',
        '--width', '1200', '--height', '480', '--output', str(output),
    ], check=True, text=True, stdout=subprocess.PIPE)
    print(rendered.stdout, end='')
    result = json.loads(next(line.split('HYPRLUME_EFFECTS_RESULT ', 1)[1]
                             for line in rendered.stdout.splitlines()
                             if line.startswith('HYPRLUME_EFFECTS_RESULT ')))
    regions = result['regions']
    assert len(regions) == 6, 'missing empty/partial/full/unavailable/invalid samples'
    assert [region['fraction'] for region in regions] == [0, 0.18, 0.5, 1, 0, 0]
    assert [region['available'] for region in regions] == [True, True, True, True, False, False]
    with Image.open(output) as frame:
        pixels = frame.convert('RGB')
        measured = []
        for region in regions:
            cx, cy, radius = (region[key] for key in ['x', 'y', 'radius'])
            assert radius > 0 and cx-radius >= 0 and cy-radius >= 0
            assert cx+radius < pixels.width and cy+radius < pixels.height
            total = water = 0
            for y in range(math.floor(cy-radius), math.ceil(cy+radius)):
                for x in range(math.floor(cx-radius), math.ceil(cx+radius)):
                    if (x+0.5-cx)**2 + (y+0.5-cy)**2 > radius**2:
                        continue
                    total += 1
                    red, green, blue = pixels.getpixel((x, y))
                    # The fixture uses mint water on a dark blue vessel.
                    # Read the output pixels, independently of the fill maths.
                    water += green > 40 and green > red*1.3 and green > blue*1.08
            fraction = water / total
            expected = region['fraction']
            tolerance = 0.005 if expected == 0 else 0.035
            assert abs(fraction-expected) <= tolerance, f'water area {fraction:.3f}, expected {expected:.3f}'
            measured.append(round(fraction, 4))
    print('LIQUID AREA PASS: '+json.dumps(measured))
    print('MOTION PASS: '+result['motion'])


if __name__ == '__main__':
    main()
