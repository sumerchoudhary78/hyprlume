# Visual assets

| Asset | Origin | Use |
| --- | --- | --- |
| `backgrounds/observatory.png` | Generated with the built-in imagegen tool from the user's supplied visual reference | Cached room wallpaper; contains no baked-in UI |
| `mascot/pilot.png` | Generated with the built-in imagegen tool | Transparent assistant cutout; alpha is preserved |
| `data/WorldMap.js` | Natural Earth 1:110m land geometry, rounded coordinates and sampled land points | Real 3D globe coastlines and points |

The prompts are preserved in [asset-prompts.md](../../docs/design/asset-prompts.md).
Generated images are local project assets. Runtime does not contact an image
service or download any asset. The original supplied reference is not bundled.
The city, globe graticule, orbital rings, terrain, icons and glass are generated
by source code; no third-party 3D model pack is required.

Natural Earth data is [public domain](https://www.naturalearthdata.com/about/terms-of-use/).
Source: [ne_110m_land.geojson](https://github.com/nvkelso/natural-earth-vector/blob/master/geojson/ne_110m_land.geojson).
Downloaded source SHA-256:
`9e0729ee253ca7d7a5c4ae9395fb1902264c5377c52e224d13dd85010e2835d9`.
The derived JavaScript includes both rounded coastlines and a 1.75-degree land
point sample. It is not intended for navigation or geographical measurements.
Qt Quick 3D and other installed runtime libraries retain their own licenses.
