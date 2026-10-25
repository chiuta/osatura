#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Build the landing page (index.html and site/data/).

Builds index.html from site/index.template.html and the UI strings in site/i18n/*.json, and writes the
Markdown documents (English and translations) to site/data/docs.<lang>.js, loaded on demand by the page.
The screenshots are read from docs/img/. Run from anywhere:

    python3 scripts/build-site.py
"""
import hashlib, importlib.util, json, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
GROUPS = [
    ("start",   ["README", "GETTING_STARTED", "FAQ", "GLOSSARY", "INDEX"]),
    ("use",     ["STUDIO", "CLI_REFERENCE", "RIGGING_GUIDE", "ANIMATION_GUIDE", "PROJECT_FORMAT", "EXPORT_FORMATS", "CI"]),
    ("build",   ["ARCHITECTURE", "API", "C_ABI", "WASM", "DEVELOPMENT", "CONTRIBUTING"]),
    ("trust",   ["LIMITATIONS", "SECURITY", "CLEANROOM", "LICENSING", "THIRD_PARTY", "PATENT_NOTES", "TRADEMARK"]),
    ("project", ["ROADMAP", "FEATURES", "CHANGELOG", "RELEASE_CHECKLIST"]),
]
TITLES = {"README": "README", "ROADMAP": "Roadmap",
          "FEATURES": "Feature matrix", "CHANGELOG": "Changelog", "CLEANROOM": "Clean-room provenance",
          "INDEX": "Documentation index"}
EXTRA_DESC = {"INDEX": ("Where to find the right page: all documents, reading paths by audience, questions and answers", "everyone")}
# THIRD_PARTY_NOTICES.md (1.4 MB of generated licence texts) is linked, not embedded.


def find(name):
    for p in (ROOT / f"{name}.md", ROOT / "docs" / f"{name}.md"):
        if p.exists():
            return p
    sys.exit(f"missing document: {name}")


def descriptions():
    d = {}
    idx = (ROOT / "docs" / "INDEX.md").read_text(encoding="utf-8")
    for m in re.finditer(r"^\| \[(\w+)\.md\]\([^)]*\) \| (.*?) \| (.*?) \|$", idx, re.M):
        d[m.group(1)] = (m.group(2), m.group(3))
    root_tab = idx.split("## Documents at the repository root")[1].split("\n## ")[0]
    for m in re.finditer(r"^\| (.*?) \| (.*?) \|$", root_tab, re.M):
        for f in re.findall(r"\[\.\./([\w.]+?)\.md\]", m.group(1)):
            d.setdefault(f, (m.group(2), ""))
    return d


def main():
    desc = descriptions()
    docs, order = {}, []
    for g, ids in GROUPS:
        for i in ids:
            p = find(i)
            md = p.read_text(encoding="utf-8")
            h1 = re.search(r"^# (.+)$", md, re.M)
            title = TITLES.get(i)
            if not title and h1:
                title = re.sub(r"^OSATURA ", "", h1.group(1).replace("`", ""))
                title = title[:1].upper() + title[1:]
            d, a = desc.get(i) or EXTRA_DESC.get(i, ("", ""))
            docs[i] = {"t": title or i, "g": g,
                       "p": str(p.relative_to(ROOT)), "d": d, "a": a, "md": md}
            order.append(i)
    spec = importlib.util.spec_from_file_location("i18n_tools", ROOT / "scripts" / "i18n.py")
    i18n = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(i18n)

    # Documents are written as separate script files, loaded on demand by index.html:
    # site/data/docs.en.js (the English documents) and site/data/docs.<lang>.js (translations).
    # Plain scripts, not fetch(), so the page also works when opened from disk.
    data = ROOT / "site" / "data"
    data.mkdir(exist_ok=True)
    for old in [*data.glob("docs.*.js"), data / "demo.js", data / "app.js"]:
        if not old.exists():
            continue
        old.unlink()
    files = {"en": "window.ARM_DATA=window.ARM_DATA||{};ARM_DATA.en=" + dump({"order": order, "docs": docs}) + ";\n"}
    for lang in i18n.LANGS:
        tr = {i: i18n.tr_path(i, lang).read_text(encoding="utf-8") for i in docs if i18n.tr_path(i, lang).exists()}
        if tr:
            files[lang] = f"window.ARM_DATA=window.ARM_DATA||{{}};ARM_DATA.{lang}=" + dump(tr) + ";\n"
    h = hashlib.sha1()
    for lang, text in sorted(files.items()):
        (data / f"docs.{lang}.js").write_text(text, encoding="utf-8")
        h.update(text.encode())

    ui = {p.stem: json.load(open(p, encoding="utf-8")) for p in sorted((ROOT / "site" / "i18n").glob("*.json"))}
    base = set(ui["en"])
    for lang, d in ui.items():
        if set(d) != base:
            sys.exit(f"site/i18n/{lang}.json: keys differ from en.json: {sorted(base ^ set(d))}")

    tpl = (ROOT / "site" / "index.template.html").read_text(encoding="utf-8")
    # The step-by-step demo data (~1 MB, written into the template by scripts/build-person-demo.mjs) goes to
    # site/data/demo.js, loaded by the page only when the demo section comes near the viewport.
    d_re = re.compile(r'(<script type="application/json" id="demo-data">)([\s\S]*?)(</script>)')
    m = d_re.search(tpl)
    if not m:
        sys.exit("site/index.template.html: no #demo-data block")
    demo = "window.ARM_DATA=window.ARM_DATA||{};ARM_DATA.demo=" + m.group(2).strip() + ";\n"
    (data / "demo.js").write_text(demo, encoding="utf-8")
    h.update(demo.encode())
    files["demo"] = demo
    tpl = tpl[:m.start()] + m.group(1) + m.group(3) + tpl[m.end():]
    out = tpl.replace("/*__I18N__*/ {}", dump(ui)).replace("/*__BUILD__*/", h.hexdigest()[:10]).replace("/*__IMGSZ__*/{}", (ROOT / "site" / "img" / "sizes.json").read_text().strip() if (ROOT / "site" / "img" / "sizes.json").exists() else "{}")
    # The page script goes to site/data/app.js (cacheable, versioned by its content hash).
    s_re = re.compile(r"<script>\n([\s\S]*?)</script>\s*(?=</body>)")
    m = s_re.search(out)
    if not m:
        sys.exit("site/index.template.html: no page <script> before </body>")
    app = m.group(1)
    (data / "app.js").write_text(app, encoding="utf-8")
    v = hashlib.sha1(app.encode()).hexdigest()[:10]
    out = out[:m.start()] + f'<script src="site/data/app.js?v={v}" defer></script>\n' + out[m.end():]
    write_pages(out, ui, demo_i18n(tpl))
    size = sum(len(t.encode()) for t in files.values()) / 1024
    print(f"index.html: {len(out)/1024:.0f} KiB; site/data: {len(files)} files, {size:.0f} KiB; {len(docs)} documents")



SITE = "https://osatura.chiuta.ro/"
CSP = ("default-src 'self'; script-src 'self' {h}; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; "
       "connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'none'")


def demo_i18n(tpl):
    m = re.search(r"/\*demo:i18n\*/Object\.entries\(([\s\S]*?)\)\.forEach", tpl)
    return json.loads(m.group(1)) if m else {}


def attr(v):
    return str(v).replace("&", "&amp;").replace('"', "&quot;").replace("<", "&lt;")


def prerender(html, t, lang, langs):
    """Static text for one language: the same values applyLang() would set, so crawlers and no-JS readers see them."""
    root = "" if lang == "en" else "../"
    url = SITE + ("" if lang == "en" else f"{lang}/")
    filled = 0

    def fill(m):
        nonlocal filled
        k = m.group(3)
        if k not in t:
            return m.group(0)
        filled += 1
        return m.group(1) + str(t[k]) + m.group(4)
    html = re.sub(r'(<(\w+)\b[^>]*\sdata-i="(\w+)"[^>]*>)(</\2>)', fill, html)
    html = re.sub(r'(\sdata-ph="(\w+)")', lambda m: m.group(1) + (f' placeholder="{attr(t[m.group(2)])}"' if m.group(2) in t else ""), html)
    html = re.sub(r'aria-label="[^"]*"(\s+data-al="(\w+)")', lambda m: f'aria-label="{attr(t[m.group(2)])}"' + m.group(1) if m.group(2) in t else m.group(0), html)
    for sid in ("lang", "rlang"):
        html = re.sub(rf'(id="{sid}"[^>]*?aria-label=")[^"]*', lambda m: m.group(1) + attr(t["alLang"] + ": " + lang.upper()), html)
    if "tiPartial" in t:
        html = html.replace('class="pt" title="partial"', f'class="pt" title="{attr(t["tiPartial"])}"')
    for key in ("vy", "vx"):
        html = re.sub(rf'(<ul\b[^>]*\sid="{key}"[^>]*>)(</ul>)', lambda m: m.group(1) + "".join(f"<li>{x}</li>" for x in t[key]) + m.group(2), html)
    html = html.replace('<html lang="en">', f'<html lang="{lang}">', 1)
    html = re.sub(r"<title>[^<]*</title>", f"<title>{attr(t['metaTitle'])}</title>", html, count=1)
    html = re.sub(r'(<meta name="description" content=")[^"]*', lambda m: m.group(1) + attr(t["metaDesc"]), html, count=1)
    html = re.sub(r'(<meta property="og:(?:title|description)" content=")[^"]*',
                  lambda m: m.group(1) + attr(t["metaTitle"] if "title" in m.group(1) else t["metaDesc"]), html)
    html = re.sub(r'(<meta name="twitter:(?:title|description)" content=")[^"]*',
                  lambda m: m.group(1) + attr(t["metaTitle"] if "title" in m.group(1) else t["metaDesc"]), html)
    html = html.replace(f'<link rel="canonical" href="{SITE}">', f'<link rel="canonical" href="{url}">', 1)
    html = html.replace(f'<meta property="og:url" content="{SITE}">', f'<meta property="og:url" content="{url}">', 1)
    alt = "".join(f'<link rel="alternate" hreflang="{l}" href="{SITE}{"" if l == "en" else l + "/"}">\n' for l in langs)
    alt += f'<link rel="alternate" hreflang="x-default" href="{SITE}">\n'
    html = html.replace('<link rel="canonical"', alt + '<link rel="canonical"', 1)
    html = html.replace("EN ▾</button>", f"{lang.upper()} ▾</button>")
    boot = f"window.OSA_ROOT='{root}'" + ("" if lang == "en" else f";window.OSA_LANG='{lang}'")
    sha = "'sha256-" + __import__("base64").b64encode(hashlib.sha256(boot.encode()).digest()).decode() + "'"
    html = html.replace("/*__CSP__*/", CSP.format(h=sha), 1)
    html = html.replace('<script src="site/data/app.js', f"<script>{boot}</script>\n" + '<script src="site/data/app.js', 1)
    if root:
        html = re.sub(r'(\s(?:src|href)=")(?=(?:site|docs|try)/)', r"\1" + root, html)
    return html, filled


def write_pages(html, ui, demo):
    langs = list(ui)
    langs.sort(key=lambda l: (l != "en", l))
    total = len(re.findall(r'\sdata-i="', html))
    for lang in langs:
        t = dict(ui["en"]); t.update(demo.get("en", {}))
        if lang != "en":
            t.update(ui[lang]); t.update(demo.get(lang, {}))
        page, filled = prerender(html, t, lang, langs)
        if filled < total:
            sys.exit(f"{lang}: only {filled} of {total} data-i elements were filled")
        dest = ROOT / ("index.html" if lang == "en" else f"{lang}/index.html")
        dest.parent.mkdir(exist_ok=True)
        dest.write_text(page, encoding="utf-8")
    urls = [SITE] + [f"{SITE}{l}/" for l in langs if l != "en"] + [f"{SITE}try/"]
    (ROOT / "sitemap.xml").write_text('<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
                                      + "".join(f"  <url><loc>{u}</loc></url>\n" for u in urls) + "</urlset>\n", encoding="utf-8")

def dump(obj):
    return json.dumps(obj, ensure_ascii=False, separators=(",", ":"))


if __name__ == "__main__":
    main()
