#!/usr/bin/env python3
"""Render a shaped backlog/context packet into a static HTML handoff artifact."""

from __future__ import annotations

import argparse
import hashlib
import html
import re
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
REQUIRED_SECTIONS = [
    "PRD Summary",
    "Goal",
    "Product Requirements",
    "Technical Design",
    "Deliverable",
    "Oracle",
    "Implementation Sequence",
    "Risk + Rollout",
]


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def slug(value: str) -> str:
    text = re.sub(r"[^a-z0-9-]+", "-", value.lower()).strip("-")
    return text or "context-packet"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def title_from(markdown: str, source: Path) -> str:
    for line in markdown.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return source.stem


def metadata(markdown: str) -> dict[str, str]:
    fields: dict[str, str] = {}
    for key in ["Priority", "Status", "Estimate"]:
        match = re.search(rf"^{key}:\s*(.+)$", markdown, re.MULTILINE)
        if match:
            fields[key.lower()] = match.group(1).strip()
    return fields


def sections(markdown: str) -> dict[str, str]:
    result: dict[str, str] = {}
    matches = list(re.finditer(r"^##\s+(.+?)\s*$", markdown, re.MULTILINE))
    for index, match in enumerate(matches):
        name = match.group(1).strip()
        start = match.end()
        end = matches[index + 1].start() if index + 1 < len(matches) else len(markdown)
        result[name] = markdown[start:end].strip()
    return result


def first_paragraph(text: str) -> str:
    chunks = [chunk.strip() for chunk in re.split(r"\n\s*\n", text) if chunk.strip()]
    return chunks[0] if chunks else ""


def key_values(text: str) -> list[tuple[str, str]]:
    pairs: list[tuple[str, str]] = []
    current_key = ""
    current_value: list[str] = []
    for raw in text.splitlines():
        line = raw.rstrip()
        match = re.match(r"^-\s+([^:]+):\s*(.+)$", line)
        if match:
            if current_key:
                pairs.append((current_key, " ".join(current_value).strip()))
            current_key = match.group(1).strip()
            current_value = [match.group(2).strip()]
        elif current_key and (raw.startswith("  ") or raw.startswith("\t")) and line.strip():
            current_value.append(line.strip())
        elif current_key and not line.strip():
            pairs.append((current_key, " ".join(current_value).strip()))
            current_key = ""
            current_value = []
    if current_key:
        pairs.append((current_key, " ".join(current_value).strip()))
    return pairs


def key_value_grid(text: str, limit: int | None = None) -> str:
    pairs = key_values(text)
    if limit is not None:
        pairs = pairs[:limit]
    if not pairs:
        return markdown_block(text)
    items = []
    for key, value in pairs:
        items.append(f"<div><dt>{markdown_inline(key)}</dt><dd>{markdown_inline(value)}</dd></div>")
    return '<dl class="spec-grid">' + "".join(items) + "</dl>"


def markdown_inline(value: str) -> str:
    escaped = html.escape(value)
    escaped = re.sub(r"`([^`]+)`", r"<code>\1</code>", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    return escaped


def markdown_block(text: str) -> str:
    if not text:
        return '<p class="muted">Not specified.</p>'
    lines = text.splitlines()
    output: list[str] = []
    list_items: list[str] = []
    table_rows: list[list[str]] = []
    paragraph: list[str] = []
    in_code = False
    code_lines: list[str] = []

    def flush_list() -> None:
        nonlocal list_items
        if list_items:
            output.append("<ul>" + "".join(f"<li>{item}</li>" for item in list_items) + "</ul>")
            list_items = []

    def flush_table() -> None:
        nonlocal table_rows
        if not table_rows:
            return
        body = []
        for row_index, row in enumerate(table_rows):
            tag = "th" if row_index == 0 else "td"
            if row_index == 1 and all(re.fullmatch(r":?-{3,}:?", cell.strip()) for cell in row):
                continue
            body.append("<tr>" + "".join(f"<{tag}>{markdown_inline(cell.strip())}</{tag}>" for cell in row) + "</tr>")
        output.append('<div class="table-wrap"><table>' + "".join(body) + "</table></div>")
        table_rows = []

    def flush_paragraph() -> None:
        nonlocal paragraph
        if paragraph:
            output.append(f"<p>{markdown_inline(' '.join(paragraph))}</p>")
            paragraph = []

    for raw in lines:
        line = raw.rstrip()
        if line.startswith("```"):
            flush_paragraph()
            flush_list()
            flush_table()
            if in_code:
                output.append("<pre><code>" + html.escape("\n".join(code_lines)) + "</code></pre>")
                code_lines = []
                in_code = False
            else:
                in_code = True
            continue
        if in_code:
            code_lines.append(line)
            continue
        if not line.strip():
            flush_paragraph()
            flush_list()
            flush_table()
            continue
        if line.startswith("|") and line.endswith("|"):
            flush_paragraph()
            flush_list()
            table_rows.append([cell for cell in line.strip("|").split("|")])
            continue
        if (raw.startswith("  ") or raw.startswith("\t")) and list_items:
            list_items[-1] = f"{list_items[-1]} {markdown_inline(line.strip())}"
            continue
        if line.startswith("- ") or re.match(r"^\d+\.\s+", line):
            flush_paragraph()
            flush_table()
            item = re.sub(r"^(-|\d+\.)\s+", "", line).strip()
            list_items.append(markdown_inline(item))
            continue
        if line.startswith("### "):
            flush_paragraph()
            flush_list()
            flush_table()
            output.append(f"<h3>{markdown_inline(line[4:].strip())}</h3>")
            continue
        paragraph.append(line.strip())
    flush_paragraph()
    flush_list()
    flush_table()
    if in_code:
        output.append("<pre><code>" + html.escape("\n".join(code_lines)) + "</code></pre>")
    return "\n".join(output)


def section_class(name: str) -> str:
    important = {"PRD Summary", "Product Requirements", "Technical Design", "Deliverable", "Oracle"}
    evidence = {"Acceptance Evidence", "Formal Spec", "Delegation Evidence", "Residual Risk", "Observability Plan"}
    if name in important:
        return "doc-section primary-section"
    if name in evidence:
        return "doc-section evidence-section"
    if "Risk" in name:
        return "doc-section risk-section"
    return "doc-section"


def section_markup(name: str, text: str) -> str:
    body = key_value_grid(text) if name in {"PRD Summary", "Product Requirements", "Technical Design", "Deliverable"} else markdown_block(text)
    return f'<section class="{section_class(name)}" id="{slug(name)}"><h2>{html.escape(name)}</h2>{body}</section>'


def render(source: Path, output: Path) -> None:
    markdown = read_text(source)
    title = title_from(markdown, source)
    packet_sections = sections(markdown)
    meta = metadata(markdown)
    missing = [name for name in REQUIRED_SECTIONS if name not in packet_sections]
    if missing:
        raise SystemExit(f"{source}: missing required shaped-doc section(s): {', '.join(missing)}")
    source_rel = source.relative_to(REPO_ROOT) if source.is_relative_to(REPO_ROOT) else source
    chips = "".join(
        f"<span><b>{html.escape(key.title())}</b> {html.escape(value)}</span>"
        for key, value in meta.items()
    )
    body_sections = []
    for name in [
        "PRD Summary",
        "Goal",
        "Shaping Decision — 2026-06-03",
        "Product Requirements",
        "Technical Design",
        "Deliverable",
        "Constraints / Invariants",
        "Authority Order",
        "Repo Anchors",
        "Alternatives Considered",
        "Tradeoff Matrix",
        "Oracle",
        "Acceptance Evidence",
        "Formal Spec",
        "Implementation Sequence",
        "Risk + Rollout",
    ]:
        if name in packet_sections:
            body_sections.append(section_markup(name, packet_sections[name]))
    review_gate = """<section class="review-gate" id="browser-inspection">
    <h2>Review Gate</h2>
    <ul>
      <li>Open this artifact in a browser after generation.</li>
      <li>Confirm hierarchy, tables, implementation steps, and long code/path text fit on desktop and mobile widths.</li>
      <li>Record the inspected file path, viewport evidence, and residual visual risk in closeout.</li>
    </ul>
  </section>
"""
    source_hash = sha256(source)
    regenerate = f"python3 skills/shape/scripts/render_context_packet_doc.py {source_rel} --output {output.relative_to(REPO_ROOT) if output.is_relative_to(REPO_ROOT) else output}"
    html_doc = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="generator" content="skills/shape/scripts/render_context_packet_doc.py">
  <meta name="harness-kit-shape-source" content="{html.escape(str(source_rel))}">
  <meta name="harness-kit-shape-source-sha256" content="{source_hash}">
  <meta name="harness-kit-shape-regenerate" content="{html.escape(regenerate)}">
  <title>{html.escape(title)} | Shape Packet</title>
  <style>
    :root {{ --ink:#111315; --muted:#5a626b; --line:#d9ded7; --paper:#f6f7f3; --panel:#ffffff; --accent:#0b6b57; --accent2:#9a6a10; --risk:#8a2f1d; }}
    * {{ box-sizing:border-box; }}
    html, body {{ overflow-x:clip; }}
    body {{ margin:0; font-family:ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color:var(--ink); background:var(--paper); line-height:1.58; }}
    main {{ max-width:1120px; margin:0 auto; padding:28px clamp(16px, 4vw, 48px) 64px; }}
    .masthead {{ border-top:6px solid var(--accent); padding:24px 0 18px; display:grid; grid-template-columns:minmax(0, 1.35fr) minmax(280px, .65fr); gap:28px; align-items:start; }}
    .eyebrow {{ margin:0 0 8px; color:var(--accent); text-transform:uppercase; letter-spacing:.08em; font-weight:850; font-size:12px; }}
    h1 {{ margin:0; max-width:820px; font-size:clamp(30px, 4.4vw, 48px); line-height:1.05; letter-spacing:0; overflow-wrap:anywhere; }}
    h2 {{ margin:0 0 12px; font-size:clamp(21px, 2.4vw, 28px); line-height:1.16; letter-spacing:0; }}
    h3 {{ margin:18px 0 8px; }}
    p {{ margin:0 0 14px; max-width:72ch; }}
    .lede {{ color:var(--muted); font-size:18px; max-width:760px; margin-top:16px; }}
    .chips {{ display:flex; flex-wrap:wrap; gap:8px; margin-top:18px; }}
    .chips span {{ display:inline-flex; align-items:center; border:1px solid var(--line); border-radius:6px; padding:6px 9px; color:var(--muted); background:#fff; font-size:13px; }}
    .decision-panel {{ min-width:0; border:1px solid var(--line); background:var(--panel); border-radius:8px; padding:18px; }}
    .decision-panel h2 {{ font-size:18px; }}
    .decision-panel p {{ color:var(--muted); }}
    .spec-grid {{ margin:0; display:grid; grid-template-columns:repeat(2, minmax(0, 1fr)); gap:12px; }}
    .spec-grid div {{ min-width:0; padding:10px 0; border-top:1px solid var(--line); }}
    .spec-grid div:nth-child(-n+2) {{ border-top:0; }}
    .spec-grid dt {{ margin:0 0 5px; color:var(--accent); font-size:12px; font-weight:850; text-transform:uppercase; letter-spacing:.06em; }}
    .spec-grid dd {{ margin:0; color:var(--ink); }}
    .doc-section, .review-gate {{ min-width:0; padding:22px 0; margin:0; border-top:1px solid var(--line); }}
    .primary-section {{ padding:26px 0; }}
    .evidence-section {{ color:#222; }}
    .risk-section h2, .review-gate h2 {{ color:var(--risk); }}
    ul {{ padding-left:22px; }}
    li + li {{ margin-top:8px; }}
    code {{ font-family:ui-monospace, SFMono-Regular, Menlo, monospace; border:1px solid #d8e3de; background:#f8fbf9; border-radius:5px; padding:2px 5px; overflow-wrap:anywhere; }}
    pre {{ overflow:auto; padding:16px; border-radius:8px; background:#101827; color:#e9eefc; }}
    pre code {{ border:0; padding:0; color:inherit; background:transparent; }}
    .table-wrap {{ max-width:100%; overflow:auto; border:1px solid var(--line); border-radius:8px; margin:14px 0; }}
    table {{ width:100%; border-collapse:collapse; min-width:720px; background:#fff; }}
    th, td {{ border-bottom:1px solid var(--line); padding:10px 12px; text-align:left; vertical-align:top; }}
    th {{ background:#eef3ef; }}
    .muted {{ color:var(--muted); }}
    .artifact-meta {{ margin-top:28px; padding-top:18px; border-top:1px solid var(--line); color:var(--muted); font-size:13px; }}
    .artifact-meta dl {{ margin:0; display:grid; gap:8px; }}
    .artifact-meta dt {{ font-weight:800; color:var(--ink); }}
    .artifact-meta dd {{ margin:0 0 6px; }}
    @media (max-width:900px) {{ .masthead, .spec-grid {{ grid-template-columns:1fr; }} .spec-grid div:nth-child(2) {{ border-top:1px solid var(--line); }} main {{ padding-top:18px; }} }}
  </style>
</head>
<body>
<main>
  <section class="masthead">
    <div>
      <p class="eyebrow">Operational PRD</p>
      <h1>{html.escape(title)}</h1>
      <p class="lede">{markdown_inline(first_paragraph(packet_sections["Goal"]))}</p>
      <div class="chips">{chips}<span><b>Source</b>{html.escape(str(source_rel))}</span></div>
    </div>
    <aside class="decision-panel">
      <h2>Deliverable</h2>
      {key_value_grid(packet_sections.get("Deliverable", ""), limit=4)}
    </aside>
  </section>
  {''.join(body_sections)}
  {review_gate}
  <section class="artifact-meta" aria-label="Artifact metadata">
    <dl>
      <div><dt>Source hash</dt><dd><code>{source_hash}</code></dd></div>
      <div><dt>Regenerate</dt><dd><code>{html.escape(regenerate)}</code></dd></div>
    </dl>
  </section>
</main>
</body>
</html>
"""
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(html_doc, encoding="utf-8")


def self_test() -> None:
    fixture = """# Context Packet: Example

Priority: P1
Status: ready
Estimate: M

## Goal

Prove the renderer creates a browsable handoff.

## PRD Summary

- User: Harness Kit operator reviewing a shaped ticket.
- Problem: Markdown packets bury product and technical decisions.
- Why now: HTML handoffs need a stricter source shape.
- UX enabled: Reviewer can scan the deliverable and oracle first.
- Deliverable type: docs artifact.
- Success signal: renderer produces the PRD sections.

## Product Requirements

- P0: Render PRD sections before implementation detail.
- P1: Preserve list continuation text across lines.
- Non-goals: Do not replace Markdown as source.

## Technical Design

- Chosen architecture: static renderer over a shaped Markdown packet.
- Files/systems touched: renderer only.
- Data/control flow: source Markdown to one HTML artifact.
- Build/check boundary: renderer fails on missing PRD sections.
- ADR decision: not required; this is a docs handoff helper.
- Design X vs Y: strict PRD source over permissive rendering.

## Oracle

- [ ] HTML exists.
- [ ] Browser gate is present.
- [ ] Wrapped list text
      continues across lines.

## Deliverable

- Output: standalone HTML handoff.
- Acceptance oracle: renderer self-test.
- Evidence artifacts: generated temporary HTML.
- Residual risk: browser inspection still requires a real viewport.

## Implementation Sequence

1. Read source.
2. Render artifact.
3. Inspect browser.

## Risk + Rollout

- Remove the artifact if it drifts.
"""
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "packet.md"
        output = Path(tmp) / "packet.html"
        source.write_text(fixture, encoding="utf-8")
        render(source, output)
        text = output.read_text(encoding="utf-8")
        for expected in [
            "Review Gate",
            "Implementation Sequence",
            "Deliverable",
            "Wrapped list text continues across lines.",
            "harness-kit-shape-source-sha256",
        ]:
            if expected not in text:
                raise AssertionError(f"missing rendered marker: {expected}")
        bad_source = Path(tmp) / "bad.md"
        bad_source.write_text("# Context Packet: Bad\n\n## Goal\n\nNo oracle.\n", encoding="utf-8")
        try:
            render(bad_source, Path(tmp) / "bad.html")
        except SystemExit:
            pass
        else:
            raise AssertionError("renderer accepted incomplete packet")
    print("shape context packet renderer self-test ok")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, help="Markdown backlog/context packet")
    parser.add_argument("--output", type=Path, help="HTML output path")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if not args.source or not args.output:
        parser.error("provide source and --output, or use --self-test")
    source = args.source if args.source.is_absolute() else REPO_ROOT / args.source
    output = args.output if args.output.is_absolute() else REPO_ROOT / args.output
    render(source, output)
    print(f"Rendered {output.relative_to(REPO_ROOT) if output.is_relative_to(REPO_ROOT) else output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
