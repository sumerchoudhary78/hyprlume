# Hyprlume website

A buildless project site, published at
<https://sumerchoudhary78.github.io/hyprlume/> by `.github/workflows/pages.yml`.
Only `site/` is deployed. It contains no native daemon code, models, credentials
or Arch-session configuration. Updating the site never deploys the desktop.

## Local preview

From the repository root:

```sh
python3 -m http.server 4173 --directory site --bind 127.0.0.1
```

Open <http://127.0.0.1:4173>. No installation, build or API key is needed.
HTML owns content, `assets/styles.css` owns layout/materials, and `assets/app.js`
adds accessible tabs, screenshot expansion, command examples and copying.
All demo commands are illustrations. The page never requests microphone access
or executes a desktop action. There are no analytics, cookies or external
runtime requests; fonts and images are hosted with the page.

## Checks and publishing

```sh
python3 -m unittest discover -s tests/site -v
node --check site/assets/app.js
```

The workflow runs the checks on pull requests. Pushes to `main` affecting the
site deploy through GitHub Pages after validation. The `github-pages` environment
records each published revision. Pages must have its source set to **GitHub
Actions** in the repository settings; no custom domain is configured.

Keep relative asset URLs so the site works at the `/hyprlume/` project path.
The canonical/social URLs, sitemap and 404 links use that public prefix.
When cutting a release, update the version/release links and supported behavior
in `index.html`; use the repository's validation notes for performance claims.

## Asset provenance

- `media/*.webp` derive from the public, synthetic-data QML screenshots in
  `assets/desktop-preview.png` and `assets/connections-preview.png`. They are
  actual renders, not browser recreations or images of a user's private session.
- `brand.svg` is the project's existing mark.
- Space Grotesk is self-hosted under the SIL Open Font License retained in
  `assets/fonts/OFL.txt`. The source is Google Fonts at commit
  `00a38a53f92aef923b9353f40128e8f4552ddae4`, Latin-subset and converted to WOFF2.

To regenerate assets after updating the screenshots, use a temporary Python
environment with `Pillow`, `fonttools` and `brotli`, then run
`python scripts/site-assets.py`. These are authoring dependencies only. Generated
assets are committed; visitors and Pages builds never download a font upstream.
