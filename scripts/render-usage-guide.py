"""Renders docs/usage.md into usage.html, the user guide inside both release zips (task 048).

Windows has no default app for `.md`, so the zips carry the guide as one self-contained HTML page that opens in
the default browser: inline CSS, no script, no remote font, light and dark through `prefers-color-scheme`. Both
release scripts call this script, so the Windows and the macOS zip hold the same page.

The renderer knows the Markdown the guide uses and nothing more: headings, paragraphs, flat bullet and numbered
lists with indented continuation lines, pipe tables, fenced code, and inline code, *emphasis*, **strong**,
[links](url) and bare http(s) URLs. Anything else (a nested list, a block quote, raw HTML, a `* ` or `+ ` bullet,
an indented code block, a rule) stops it with the line number, so a guide edit that would come out wrong fails
the CI check instead of shipping. It needs nothing beyond the standard library.

    python -I scripts/render-usage-guide.py --out dist/usage.html   # docs/usage.md → dist/usage.html
    python -I scripts/render-usage-guide.py --check                 # self-test, then docs/usage.md in memory
"""

from __future__ import annotations

import argparse
import html
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUIDE = ROOT / "docs" / "usage.md"

HEADING = re.compile(r"^(#{1,6})\s+(.+?)\s*#*\s*$")
BULLET = re.compile(r"^- (.*)$")
NUMBERED = re.compile(r"^(\d+)\. (.*)$")
FENCE = re.compile(r"^```")
TABLE_RULE = re.compile(r"^\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)*\|?\s*$")
URL = re.compile(r"https?://[^\s<>`]+")
LINK = re.compile(r"\[([^\]]+)\]\(([^)\s]+)\)")
STRONG = re.compile(r"\*\*(?=\S)(.+?)(?<=\S)\*\*")
EMPHASIS = re.compile(r"\*(?=\S)(.+?)(?<=\S)\*")

CSS = """
:root { color-scheme: light dark; --bg: #ffffff; --fg: #1f2328; --muted: #59636e; --line: #d1d9e0;
  --code-bg: #eff2f5; --link: #0969da; --head-bg: #f6f8fa; }
@media (prefers-color-scheme: dark) {
  :root { --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --line: #3d444d;
    --code-bg: #1f262e; --link: #4493f8; --head-bg: #151b23; }
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg);
  font: 16px/1.6 -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif; }
main { max-width: 52rem; margin: 0 auto; padding: 2rem 1.25rem 4rem; }
h1 { font-size: 1.9rem; line-height: 1.25; margin: 0 0 1rem; }
h2 { font-size: 1.35rem; margin: 2.2rem 0 0.6rem; padding-bottom: 0.3rem; border-bottom: 1px solid var(--line); }
h3, h4, h5, h6 { font-size: 1.1rem; margin: 1.6rem 0 0.5rem; }
p, ul, ol, table, pre { margin: 0 0 1rem; }
ul, ol { padding-left: 1.6rem; }
li + li { margin-top: 0.3rem; }
a { color: var(--link); overflow-wrap: anywhere; }
code { font: 0.88em/1.4 ui-monospace, SFMono-Regular, Consolas, Menlo, monospace; background: var(--code-bg);
  padding: 0.1em 0.35em; border-radius: 4px; }
pre { background: var(--code-bg); padding: 0.8rem 1rem; border-radius: 6px; overflow-x: auto; }
pre code { padding: 0; background: none; }
table { border-collapse: collapse; width: 100%; display: block; overflow-x: auto; }
th, td { border: 1px solid var(--line); padding: 0.4rem 0.7rem; text-align: left; vertical-align: top; }
th { background: var(--head-bg); }
em { font-style: italic; }
""".strip()


class GuideError(ValueError):
    """A construct the renderer does not support, with its 1-based line number."""

    def __init__(self, line: int, message: str):
        super().__init__(f"line {line}: {message}")


# --- inline -------------------------------------------------------------------------------


def _trim_url(url: str) -> tuple[str, str]:
    """A bare URL without the sentence punctuation after it: `.`, `,` … and a `)` that closes text around it."""
    tail = ""
    while url:
        last = url[-1]
        if last in ".,;:!?" or (last == ")" and url.count("(") < url.count(")")):
            tail = last + tail
            url = url[:-1]
        else:
            break
    return url, tail


def _emphasis(text: str) -> str:
    """Escaped text with **strong** and *emphasis*."""
    text = html.escape(text, quote=False)
    text = STRONG.sub(r"<strong>\1</strong>", text)
    return EMPHASIS.sub(r"<em>\1</em>", text)


def _anchor(url: str, label_html: str) -> str:
    return f'<a href="{html.escape(url, quote=True)}">{label_html}</a>'


def _text(text: str) -> str:
    """Text outside code spans: links and bare URLs become anchors, the rest gets emphasis."""
    out: list[str] = []
    pos = 0
    while pos < len(text):
        link = LINK.search(text, pos)
        url = URL.search(text, pos)
        nxt = min((m for m in (link, url) if m), key=lambda m: m.start(), default=None)
        if nxt is None:
            break
        out.append(_emphasis(text[pos:nxt.start()]))
        if nxt is link:
            out.append(_anchor(link.group(2), _emphasis(link.group(1))))
        else:
            bare, tail = _trim_url(url.group(0))
            out.append(_anchor(bare, html.escape(bare, quote=False)) + html.escape(tail, quote=False))
        pos = nxt.end()
    out.append(_emphasis(text[pos:]))
    return "".join(out)


def inline(text: str, line: int) -> str:
    """One line or a joined paragraph: code spans first, so nothing inside backticks is interpreted."""
    parts = text.split("`")
    if len(parts) % 2 == 0:
        raise GuideError(line, "unclosed code span (odd number of backticks)")
    return "".join(
        f"<code>{html.escape(part, quote=False)}</code>" if i % 2 else _text(part) for i, part in enumerate(parts)
    )


# --- blocks -------------------------------------------------------------------------------


def _cells(row: str) -> list[str]:
    """The cells of a pipe-table row; a `|` inside a code span belongs to the cell."""
    row = row.strip()
    if row.startswith("|"):
        row = row[1:]
    if row.endswith("|"):
        row = row[:-1]
    cells, cell, in_code = [], [], False
    for ch in row:
        if ch == "`":
            in_code = not in_code
        if ch == "|" and not in_code:
            cells.append("".join(cell).strip())
            cell = []
        else:
            cell.append(ch)
    cells.append("".join(cell).strip())
    return cells


def _slug(text: str, used: set[str]) -> str:
    base = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-") or "section"
    slug, n = base, 2
    while slug in used:
        slug, n = f"{base}-{n}", n + 1
    used.add(slug)
    return slug


def _unsupported(line: str) -> str | None:
    """Why a line that starts a block cannot be rendered, or None."""
    if line.startswith(">"):
        return "block quotes are not supported"
    if line.startswith("<"):
        return "raw HTML is not supported"
    if re.match(r"^[*+] ", line):
        return "use `- ` for bullets"
    if re.match(r"^(\*\s*){3,}$|^(-\s*){3,}$|^(_\s*){3,}$", line):
        return "horizontal rules are not supported"
    if line.startswith("    ") or line.startswith("\t"):
        return "indented code blocks are not supported; use a ``` fence"
    return None


def render_body(markdown: str) -> tuple[str, str]:
    """The guide's HTML body and its title (the first heading)."""
    lines = markdown.replace("\r\n", "\n").split("\n")
    out: list[str] = []
    title = ""
    used: set[str] = set()
    i = 0

    def blank(k: int) -> bool:
        return k >= len(lines) or not lines[k].strip()

    def starts_block(k: int) -> bool:
        s = lines[k]
        return bool(
            HEADING.match(s) or BULLET.match(s) or NUMBERED.match(s) or FENCE.match(s) or s.startswith("|")
            or _unsupported(s)
        )

    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        if reason := _unsupported(line):
            raise GuideError(i + 1, reason)

        if m := HEADING.match(line):
            level, text = len(m.group(1)), m.group(2)
            if not title:
                title = text
            body = inline(text, i + 1)
            if level == 1:
                out.append(f"<h1>{body}</h1>")
            else:
                out.append(f'<h{level} id="{_slug(text, used)}">{body}</h{level}>')
            i += 1
            continue

        if FENCE.match(line):
            start = i
            i += 1
            code: list[str] = []
            while i < len(lines) and not FENCE.match(lines[i]):
                code.append(lines[i])
                i += 1
            if i == len(lines):
                raise GuideError(start + 1, "unclosed ``` fence")
            out.append("<pre><code>" + html.escape("\n".join(code), quote=False) + "</code></pre>")
            i += 1
            continue

        if line.startswith("|"):
            if i + 1 >= len(lines) or not TABLE_RULE.match(lines[i + 1]):
                raise GuideError(i + 1, "a table needs a |---| rule under its header row")
            header = _cells(line)
            rows: list[list[str]] = []
            i += 2
            while i < len(lines) and lines[i].startswith("|"):
                cells = _cells(lines[i])
                if len(cells) != len(header):
                    raise GuideError(i + 1, f"{len(cells)} cells, the header has {len(header)}")
                rows.append(cells)
                i += 1
            head = "".join(f"<th>{inline(c, i)}</th>" for c in header)
            body = "".join("<tr>" + "".join(f"<td>{inline(c, i)}</td>" for c in r) + "</tr>" for r in rows)
            out.append(f"<table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>")
            continue

        if BULLET.match(line) or NUMBERED.match(line):
            numbered = bool(NUMBERED.match(line))
            marker = NUMBERED if numbered else BULLET
            first = NUMBERED.match(line)
            start_no = int(first.group(1)) if first else 1
            items: list[tuple[int, list[str]]] = []
            while i < len(lines) and not blank(i):
                s = lines[i]
                if m := marker.match(s):
                    items.append((i + 1, [m.group(m.lastindex)]))
                elif s.startswith(" "):
                    stripped = s.strip()
                    if BULLET.match(stripped) or NUMBERED.match(stripped) or re.match(r"^[*+] ", stripped):
                        raise GuideError(i + 1, "nested lists are not supported")
                    items[-1][1].append(stripped)
                elif (BULLET if numbered else NUMBERED).match(s):
                    raise GuideError(i + 1, "a bullet and a numbered list need a blank line between them")
                else:
                    raise GuideError(i + 1, "a list item's continuation line must be indented")
                i += 1
            tag = "ol" if numbered else "ul"
            attr = f' start="{start_no}"' if numbered and start_no != 1 else ""
            lis = "".join(f"<li>{inline(' '.join(parts), no)}</li>" for no, parts in items)
            out.append(f"<{tag}{attr}>{lis}</{tag}>")
            continue

        start = i
        para: list[str] = []
        while i < len(lines) and not blank(i) and (i == start or not starts_block(i)):
            para.append(lines[i].strip())
            i += 1
        out.append(f"<p>{inline(' '.join(para), start + 1)}</p>")

    if not title:
        raise GuideError(1, "the guide needs a heading for its title")
    return "\n".join(out), title


def render(markdown: str) -> str:
    """The whole page."""
    body, title = render_body(markdown)
    plain_title = re.sub(r"<[^>]+>", "", inline(title, 1))
    return (
        "<!doctype html>\n"
        '<html lang="en">\n<head>\n<meta charset="utf-8">\n'
        '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
        f"<title>{plain_title}</title>\n<style>\n{CSS}\n</style>\n</head>\n"
        f"<body>\n<main>\n{body}\n</main>\n</body>\n</html>\n"
    )


# --- self-test ----------------------------------------------------------------------------


def self_test() -> None:
    """Every construct the guide uses, and the errors for the ones it must not."""
    sample = "\n".join(
        [
            "# mboxer — user guide",
            "",
            "A *viewer* with **strong** text, `code <b>&`, a [link](https://example.com/a?b=1&c=2) and",
            "a bare URL (https://example.com/x), then https://example.com/y.",
            "",
            "## What you need",
            "",
            "- one item with `- not a nested list`",
            "  continued here",
            "- *Open…* (`Ctrl+O`)",
            "",
            "3. third",
            "4. fourth",
            "",
            "| What | Windows |",
            "|---|---|",
            "| `a|b` | `%TEMP%\\mboxer\\` |",
            "",
            "```",
            "<raw> & code",
            "```",
        ]
    )
    page = render(sample)
    expected = [
        "<title>mboxer — user guide</title>",
        "<h1>mboxer — user guide</h1>",
        '<h2 id="what-you-need">What you need</h2>',
        "<em>viewer</em>",
        "<strong>strong</strong>",
        "<code>code &lt;b&gt;&amp;</code>",
        '<a href="https://example.com/a?b=1&amp;c=2">link</a>',
        '(<a href="https://example.com/x">https://example.com/x</a>)',
        '<a href="https://example.com/y">https://example.com/y</a>.',
        "<li>one item with <code>- not a nested list</code> continued here</li>",
        "<li><em>Open…</em> (<code>Ctrl+O</code>)</li>",
        '<ol start="3"><li>third</li><li>fourth</li></ol>',
        "<td><code>a|b</code></td>",
        "<td><code>%TEMP%\\mboxer\\</code></td>",
        "<pre><code>&lt;raw&gt; &amp; code</code></pre>",
        "@media (prefers-color-scheme: dark)",
    ]
    missing = [e for e in expected if e not in page]
    if missing:
        raise AssertionError("self-test: missing from the page:\n  " + "\n  ".join(missing))
    for bad, why in [
        ("# t\n\n- a\n  - b", "nested lists"),
        ("# t\n\n> quote", "block quotes"),
        ("# t\n\n* star", "use `- `"),
        ("# t\n\n| a |\n| b |", "rule under its header"),
        ("# t\n\nodd ` tick", "unclosed code span"),
        ("# t\n\n```\nnever closed", "unclosed ``` fence"),
        ("# t\n\n---", "horizontal rules"),
    ]:
        try:
            render(bad)
        except GuideError as err:
            if why not in str(err):
                raise AssertionError(f"self-test: {bad!r} failed with {err}, expected {why!r}") from None
        else:
            raise AssertionError(f"self-test: {bad!r} rendered, expected an error about {why!r}")


# --- command line -------------------------------------------------------------------------


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--guide", type=Path, default=GUIDE, help="the Markdown guide (default: docs/usage.md)")
    parser.add_argument("--out", type=Path, help="where the HTML page goes")
    parser.add_argument("--check", action="store_true", help="run the self-test and render the guide in memory")
    args = parser.parse_args()
    if not args.check and not args.out:
        parser.error("give --out or --check")

    try:
        if args.check:
            self_test()
        page = render(args.guide.read_text(encoding="utf-8"))
    except (GuideError, AssertionError) as err:
        print(f"{args.guide}: {err}", file=sys.stderr)
        return 1

    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(page, encoding="utf-8", newline="\n")
        print(f"usage guide: {args.out} ({len(page.encode('utf-8')) // 1024} KB)")
    else:
        print(f"{args.guide.name} renders ({len(page.encode('utf-8')) // 1024} KB); self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
