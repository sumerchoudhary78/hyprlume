#!/usr/bin/env python3
"""Refresh the website's checked-in images/font; never needed to serve the site.

Development dependencies: Pillow, fonttools and brotli. This only derives WebP
screenshots from public repo assets and downloads the pinned, OFL-licensed font.
"""
from io import BytesIO
from pathlib import Path
import shutil
from urllib.request import urlopen

from PIL import Image
from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "site/assets"
FONT_REVISION = "00a38a53f92aef923b9353f40128e8f4552ddae4"
FONT_ORIGIN = f"https://raw.githubusercontent.com/google/fonts/{FONT_REVISION}/ofl/spacegrotesk"


def main():
    media = ASSETS / "media"
    fonts = ASSETS / "fonts"
    media.mkdir(parents=True, exist_ok=True)
    fonts.mkdir(parents=True, exist_ok=True)
    screenshots = {
        "desktop": ROOT / "docs/design/shell-preview.webp",
        "connections": ROOT / "assets/connections-preview.png",
    }
    for name, source in screenshots.items():
        with Image.open(source) as original:
            image = original.convert("RGB")
            for width in (800, 1600):
                height = round(image.height * width / image.width)
                output = media / f"{name}-{width}.webp"
                image.resize((width, height), Image.Resampling.LANCZOS).save(
                    output, "WEBP", quality=90, method=6
                )
                print(f"{output.relative_to(ROOT)}: {output.stat().st_size:,} bytes")
    shutil.copyfile(ROOT / "assets/hyprlume.svg", ASSETS / "brand.svg")
    with urlopen(FONT_ORIGIN + "/SpaceGrotesk%5Bwght%5D.ttf", timeout=30) as response:
        font = TTFont(BytesIO(response.read()))
    options = subset.Options()
    options.flavor = "woff2"
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=[*range(0x20, 0x100), *range(0x2000, 0x2070),
                                *range(0x2190, 0x21A0), 0x2318, 0x25C7, 0x2733])
    subsetter.subset(font)
    font.flavor = "woff2"
    font.save(fonts / "space-grotesk-latin.woff2")
    with urlopen(FONT_ORIGIN + "/OFL.txt", timeout=30) as response:
        (fonts / "OFL.txt").write_bytes(response.read())
    print("Saved the pinned Space Grotesk Latin subset and its OFL notice.")


if __name__ == "__main__":
    main()
