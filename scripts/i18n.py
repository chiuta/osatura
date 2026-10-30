#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Tools for the translated documentation.

Layout
    README.<lang>.md                 translation of README.md (README.ro.md predates these tools)
    docs/i18n/<lang>/<NAME>.md       translation of docs/<NAME>.md or of a root document

Commands
    python3 scripts/i18n.py check [LANG ...]       compare each translation with its English source
    python3 scripts/i18n.py fix-links [LANG ...]   rewrite relative links and anchors for the translated location
    python3 scripts/i18n.py status                 table of translated documents per language

A translation keeps the structure of the source: same headings in the same order, byte-identical fenced
code blocks, the same inline code spans, the same link targets. `check` enforces that; `fix-links` then
points the links at the translated sibling (or at the English file when none exists) and maps heading
anchors to the translated headings. See docs/i18n/README.md for the translator guidelines.
"""
import importlib.util, os, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
LANGS = ["ro", "fr", "it", "es", "pt", "de", "hu"]


def _doc_table():
    spec = importlib.util.spec_from_file_location("build_site", ROOT / "scripts" / "build-site.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    out = {}
    for _g, ids in m.GROUPS:
        for i in ids:
            out[i] = m.find(i).relative_to(ROOT)
    return out


DOCS = _doc_table()            # id -> source path relative to the repository root
BY_PATH = {str(p): i for i, p in DOCS.items()}


def tr_path(doc_id, lang):
    if doc_id == "README":
        return ROOT / f"README.{lang}.md"
    return ROOT / "docs" / "i18n" / lang / f"{doc_id}.md"


# ---------------------------------------------------------------- markdown scanning
FENCE = re.compile(r"^\s*(`{3,}|~{3,})")


def split_fences(text):
    """Yield (is_code, lines) segments; fence lines belong to the code segment."""
    seg, code, fence = [], False, None
    for ln in text.split("\n"):
        m = FENCE.match(ln)
        if not code and m:
            if seg:
                yield False, seg
            seg, code, fence = [ln], True, m.group(1)
        elif code:
            seg.append(ln)
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= len(fence) and re.match(r"^\s*[`~]+\s*$", ln):
                yield True, seg
                seg, code = [], False
        else:
            seg.append(ln)
    if seg:
        yield code, seg


def plain(s):
    s = re.sub(r"!\[[^\]]*\]\([^)]*\)", "", s)
    s = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", s)
    s = re.sub(r"[`*]", "", s)
    s = re.sub(r"(^|\W)_(.+?)_(?=\W|$)", r"\1\2", s)
    return s.strip()


def slugify(s):
    return re.sub(r"\s", "-", re.sub(r"[^\w\- ]", "", s.lower()).strip())


def headings(text):
    """[(level, plain_text, slug)] outside code fences, with GitHub-style de-duplication."""
    out, seen = [], {}
    for code, lines in split_fences(text):
        if code:
            continue
        for ln in lines:
            m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", ln)
            if m:
                t = plain(m.group(2))
                s = slugify(t)
                n = seen.get(s, -1) + 1
                seen[s] = n
                out.append((len(m.group(1)), t, s if n == 0 else f"{s}-{n}"))
    return out


def code_blocks(text):
    return ["\n".join(lines) for code, lines in split_fences(text) if code]


def prose(text):
    return "\n".join("\n".join(lines) for code, lines in split_fences(text) if not code)


def backtick_runs(text):
    return [len(m.group(0)) for m in re.finditer(r"`+", prose(text))]


def inline_code(text):
    """Single-word inline code spans (commands, options, identifiers, file names), sorted.
    Spans that contain spaces are not compared one by one: a stray backtick in a source table row can
    shift the pairing of the spans in the lines around it. backtick_runs() still covers every backtick."""
    return sorted(re.findall(r"(?<!`)`([^`\s]+)`(?!`)", prose(text)))


def norm_target(t):
    """A link target reduced to what a translation must keep: external URL, same-page anchor, or file name."""
    if re.match(r"^(https?:|mailto:)", t):
        return t
    path, _, anchor = t.partition("#")
    if not path:
        return "#"
    base = os.path.basename(path.rstrip("/")) + ("/" if path.endswith("/") else "")
    return re.sub(r"\.(?:" + "|".join(LANGS) + r")\.md$", ".md", base)


def link_targets(text):
    return [norm_target(m.group(1)) for m in re.finditer(r"\]\(([^)\s]+)", prose(text))]


def table_rows(text):
    return sum(1 for ln in prose(text).split("\n") if ln.lstrip().startswith("|"))


def list_items(text):
    return sum(1 for ln in prose(text).split("\n") if re.match(r"^\s*([-*+]|\d+[.)])\s", ln))


def check_pair(src, tr):
    """Return a list of problems (empty when the translation preserves the structure)."""
    bad = []
    hs, ht = headings(src), headings(tr)
    if [h[0] for h in hs] != [h[0] for h in ht]:
        bad.append(f"headings: {len(hs)} in source, {len(ht)} in translation, or the levels differ")
    cs, ct = code_blocks(src), code_blocks(tr)
    if len(cs) != len(ct):
        bad.append(f"fenced code blocks: {len(cs)} in source, {len(ct)} in translation")
    else:
        for k, (a, b) in enumerate(zip(cs, ct)):
            if a != b:
                bad.append(f"code block #{k + 1} differs from the source (it must be copied verbatim)")
                break
    if backtick_runs(src) != backtick_runs(tr):
        bad.append("backtick runs differ: a code span was added, removed or altered")
    a, b = inline_code(src), inline_code(tr)
    if a != b:
        sa, sb = set(a) - set(b), set(b) - set(a)
        bad.append(f"inline code spans differ: missing {sorted(sa)[:6]}, unexpected {sorted(sb)[:6]}")
    a, b = link_targets(src), link_targets(tr)
    if a != b:
        bad.append(f"link targets differ ({len(a)} in source, {len(b)} in translation)")
    if table_rows(src) != table_rows(tr):
        bad.append(f"table rows: {table_rows(src)} in source, {table_rows(tr)} in translation")
    if list_items(src) != list_items(tr):
        bad.append(f"list items: {list_items(src)} in source, {list_items(tr)} in translation")
    keep = lambda t: [x for x in prose(t).split("\n") if x.strip() and not x.lstrip().startswith("|")]
    sl, tl = keep(src), keep(tr)
    if len(sl) > 8 and sum(1 for x in tl if x in set(sl)) > 0.6 * len(tl):
        bad.append("most prose lines are identical to the source: the text does not look translated")
    return bad


# ---------------------------------------------------------------- link fixing
LINK = re.compile(r"(!?\[(?:[^\[\]]|\[[^\]]*\])*\]\()([^)\s]+)((?:\s+\"[^\"]*\")?\))")


def fix_text(doc_id, lang, tr):
    src_path = DOCS[doc_id]
    dest = tr_path(doc_id, lang)
    src_text = (ROOT / src_path).read_text(encoding="utf-8")
    en_slugs = [h[2] for h in headings(src_text)]
    own = [h[2] for h in headings(tr)]
    amap = dict(zip(en_slugs, own)) if len(en_slugs) == len(own) else {}
    cache = {}

    def slugs_for(i):
        if i not in cache:
            t = tr_path(i, lang)
            s = (ROOT / DOCS[i]).read_text(encoding="utf-8")
            cache[i] = (dict(zip([h[2] for h in headings(s)], [h[2] for h in headings(t.read_text(encoding="utf-8"))]))
                        if t.exists() and len(headings(s)) == len(headings(t.read_text(encoding="utf-8"))) else {})
        return cache[i]

    by_base = {pathlib.PurePath(str(p)).name: str(p) for p in DOCS.values()}

    def semantic_path(p):
        """The repository-relative path a link points at, whether it is still written relative to the
        source document (as translators copy it) or was already rewritten for the translated location."""
        here = os.path.normpath(os.path.join(dest.parent, p))
        rel = os.path.relpath(here, ROOT)
        if not rel.startswith("..") and os.path.exists(here):
            return rel                                    # already valid from the translated location
        stripped = re.sub(r"^(?:\.{1,2}/)+", "", p)
        if stripped.startswith("img/"):
            return "docs/" + stripped
        if os.path.exists(ROOT / stripped):
            return stripped
        if os.path.exists(ROOT / "docs" / stripped):
            return "docs/" + stripped
        if stripped in by_base:                           # a document, however many ../ were stacked in front
            return by_base[stripped]
        return os.path.normpath(os.path.join(os.path.dirname(src_path), p))   # as written in the source

    def fix_target(t):
        if re.match(r"^(https?:|mailto:)", t):
            return t
        if t.startswith("#"):
            return "#" + amap.get(t[1:], t[1:])
        p, _, a = t.partition("#")
        if not p:
            return t
        is_dir = p.endswith("/")
        target = semantic_path(p.rstrip("/") if is_dir else p)
        m = re.match(r"docs/i18n/[a-z]{2}/([^/]+)\.md$", target)
        did = BY_PATH.get(target) or (m.group(1) if m and m.group(1) in DOCS else None)
        if not did:
            m = re.match(r"README\.([a-z]{2})\.md$", target)
            did = "README" if m else None
        if did and tr_path(did, lang).exists():
            new = os.path.relpath(tr_path(did, lang), dest.parent)
            a = slugs_for(did).get(a, a) if a else a
        else:
            new = os.path.relpath(ROOT / (str(DOCS[did]) if did else target), dest.parent)
        if is_dir and not new.endswith("/"):
            new += "/"
        return new + ("#" + a if a else "")

    def fix_link(head, t):
        other = re.search(r"\[README\.([a-z]{2})\.md\]\($", head)      # an explicit link to another language's README
        if other and other.group(1) != lang and not t.startswith(("http", "#")):
            return os.path.relpath(ROOT / f"README.{other.group(1)}.md", dest.parent)
        return fix_target(t)

    out = []
    for code, lines in split_fences(tr):
        if code:
            out.append("\n".join(lines))
        else:
            seg = "\n".join(lines)
            out.append(LINK.sub(lambda m: m.group(1) + fix_link(m.group(1), m.group(2)) + m.group(3), seg))
    return "\n".join(out)


# ---------------------------------------------------------------- commands
def langs_arg(args):
    return args or LANGS


def cmd_check(args):
    fails = 0
    for lang in langs_arg(args):
        for i, p in DOCS.items():
            t = tr_path(i, lang)
            if not t.exists() or t.name == "README.ro.md":
                continue
            bad = check_pair((ROOT / p).read_text(encoding="utf-8"), t.read_text(encoding="utf-8"))
            for b in bad:
                print(f"{t.relative_to(ROOT)}: {b}")
            fails += bool(bad)
    print("check:", "OK" if not fails else f"{fails} translation(s) with problems")
    return 1 if fails else 0


def cmd_fix_links(args):
    for lang in langs_arg(args):
        for i in DOCS:
            t = tr_path(i, lang)
            if not t.exists() or t.name == "README.ro.md":
                continue
            old = t.read_text(encoding="utf-8")
            new = fix_text(i, lang, old)
            if new != old:
                t.write_text(new, encoding="utf-8")
                print("fixed", t.relative_to(ROOT))


def cmd_status(_):
    print("| Document | " + " | ".join(LANGS) + " |")
    print("|---|" + "---|" * len(LANGS))
    for i in DOCS:
        print(f"| {i} | " + " | ".join("✓" if tr_path(i, l).exists() else "–" for l in LANGS) + " |")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    fn = {"check": cmd_check, "fix-links": cmd_fix_links, "status": cmd_status}.get(cmd)
    if not fn:
        sys.exit(__doc__)
    sys.exit(fn(sys.argv[2:]) or 0)
