"""Buildless site contracts: resources, accessibility hooks and Pages subpaths."""
from html.parser import HTMLParser
from pathlib import Path
import re
import unittest
from urllib.parse import unquote, urlsplit
import xml.etree.ElementTree as ET

SITE = Path(__file__).resolve().parents[2] / "site"
CANONICAL = "https://sumerchoudhary78.github.io/hyprlume/"


class Document(HTMLParser):
    def __init__(self, path):
        super().__init__(convert_charrefs=True)
        self.elements = []
        self.feed(path.read_text())

    def handle_starttag(self, tag, attrs):
        self.elements.append((tag, dict(attrs)))


def local_path(base, url):
    path = unquote(urlsplit(url).path)
    if path.startswith("/hyprlume/"):
        return SITE / path.removeprefix("/hyprlume/")
    return (base / path).resolve()


class WebsiteTests(unittest.TestCase):
    def test_all_local_assets_and_fragments_exist(self):
        for page in SITE.glob("*.html"):
            document = Document(page)
            ids = [attrs["id"] for _, attrs in document.elements if "id" in attrs]
            self.assertEqual(len(ids), len(set(ids)), f"Duplicate id in {page.name}")
            for tag, attrs in document.elements:
                urls = [attrs[key] for key in ("href", "src") if key in attrs]
                urls += [part.strip().split()[0] for part in attrs.get("srcset", "").split(",") if part.strip()]
                for url in urls:
                    parsed = urlsplit(url)
                    if parsed.scheme or parsed.netloc:
                        self.assertEqual(parsed.scheme, "https", url)
                        continue
                    if not parsed.path:
                        if parsed.fragment:
                            self.assertIn(parsed.fragment, ids, f"Broken anchor {url}")
                        continue
                    target = local_path(page.parent, url)
                    self.assertTrue(target.is_relative_to(SITE.resolve()), url)
                    self.assertTrue(target.exists(), f"Missing {url} in {page.name}")
                    if tag == "img":
                        self.assertIn("alt", attrs, url)
                        self.assertGreater(int(attrs.get("width", 0)), 0)
                        self.assertGreater(int(attrs.get("height", 0)), 0)

    def test_css_fonts_and_images_exist(self):
        css = SITE / "assets/styles.css"
        for url in re.findall(r'url\([\"\']?([^\)\"\']+)', css.read_text()):
            self.assertTrue(local_path(css.parent, url).is_file(), url)
        self.assertTrue((SITE / "assets/fonts/OFL.txt").is_file())

    def test_tabs_and_navigation_have_valid_targets(self):
        document = Document(SITE / "index.html")
        elements = {attrs["id"]: attrs for _, attrs in document.elements if "id" in attrs}
        for tag, attrs in document.elements:
            if attrs.get("role") == "tab":
                panel = elements[attrs["aria-controls"]]
                self.assertEqual(panel["role"], "tabpanel")
                self.assertEqual(panel["aria-labelledby"], attrs["id"])
            if tag == "button":
                self.assertEqual(attrs.get("type"), "button")
                if "aria-controls" in attrs:
                    self.assertIn(attrs["aria-controls"], elements)
        self.assertEqual(elements["site-nav"]["aria-label"], "Main navigation")

    def test_canonical_and_social_metadata_match_the_pages_url(self):
        document = Document(SITE / "index.html")
        canonical = [attrs["href"] for tag, attrs in document.elements
                     if tag == "link" and attrs.get("rel") == "canonical"]
        self.assertEqual(canonical, [CANONICAL])
        meta = {attrs.get("property", attrs.get("name")): attrs.get("content")
                for tag, attrs in document.elements if tag == "meta"}
        self.assertEqual(meta["og:url"], CANONICAL)
        self.assertTrue(meta["og:image"].startswith(CANONICAL))
        self.assertGreater(len(meta["description"]), 50)
        sitemap = ET.parse(SITE / "sitemap.xml")
        locations = [node.text for node in sitemap.iter() if node.tag.endswith("}loc")]
        self.assertEqual(locations, [CANONICAL])
        self.assertIn(CANONICAL + "sitemap.xml", (SITE / "robots.txt").read_text())

    def test_runtime_resources_are_self_hosted(self):
        document = Document(SITE / "index.html")
        for tag, attrs in document.elements:
            if tag == "script" or tag == "link" and attrs.get("rel") in ("stylesheet", "preload"):
                url = attrs.get("src", attrs.get("href", ""))
                self.assertFalse(urlsplit(url).netloc, url)
            self.assertFalse(any(key.startswith("on") for key in attrs), "Inline event handler")
        self.assertTrue((SITE / ".nojekyll").exists())

    def test_static_assets_stay_within_the_project_budget(self):
        # Includes both desktop resolutions and both gallery views, not just first load.
        assets = sum(path.stat().st_size for path in (SITE / "assets").rglob("*") if path.is_file())
        self.assertLess(assets, 2_000_000, "Recheck website image/font costs before raising this budget")
        self.assertLess((SITE / "assets/app.js").stat().st_size, 12_000)


if __name__ == "__main__":
    unittest.main()
