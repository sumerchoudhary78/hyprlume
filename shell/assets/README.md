# Visual assets

| Asset | Origin | Use |
| --- | --- | --- |
| `backgrounds/tidal-eclipse.jpg` | Generated with the built-in imagegen tool and encoded as JPEG | Default desktop wallpaper |
| `backgrounds/observatory.png` | Generated from the user's supplied visual reference | Original observatory wallpaper |
| `mascot/pilot.png` | Generated with the built-in imagegen tool | Original transparent assistant art |
| `data/WorldMap.js` | Natural Earth 1:110m land geometry, rounded coordinates and sampled land points | Original globe asset |

The prompts are preserved in [asset-prompts.md](../../docs/design/asset-prompts.md).
Generated images are bundled project assets and load locally at runtime.

Natural Earth data is [public domain](https://www.naturalearthdata.com/about/terms-of-use/).
Source: [ne_110m_land.geojson](https://github.com/nvkelso/natural-earth-vector/blob/master/geojson/ne_110m_land.geojson).
Downloaded source SHA-256:
`9e0729ee253ca7d7a5c4ae9395fb1902264c5377c52e224d13dd85010e2835d9`.
The derived JavaScript includes both rounded coastlines and a 1.75-degree land
point sample. It is not intended for navigation or geographical measurements.
Installed runtime libraries retain their own licenses.

`shaders/*.frag` are project-authored optical glass and analytic water shaders,
under the application source license. Their `.qsb` packages include SPIR-V,
GLSL, HLSL and Metal variants using Qt 6.4 serialization for compatibility with
the supported Qt 6.9+ runtimes. Compile with `python3 scripts/build-shaders.py`
and verify source/binary hashes with `--check`. The AUR package bakes these at
build time; the running shell loads bytecode without invoking a compiler tool.
