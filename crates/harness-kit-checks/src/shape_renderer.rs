use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use regex::Regex;
use sha2::{Digest, Sha256};

const REQUIRED_SECTIONS: &[&str] = &[
    "PRD Summary",
    "Goal",
    "Product Requirements",
    "Technical Design",
    "Deliverable",
    "Oracle",
    "Implementation Sequence",
    "Risk + Rollout",
];

const BODY_SECTION_ORDER: &[&str] = &[
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
];

const SPEC_GRID_SECTIONS: &[&str] = &[
    "PRD Summary",
    "Product Requirements",
    "Technical Design",
    "Deliverable",
];

const STYLE: &str = r#"
    :root { --ink:#111315; --muted:#5a626b; --line:#d9ded7; --paper:#f6f7f3; --panel:#ffffff; --accent:#0b6b57; --accent2:#9a6a10; --risk:#8a2f1d; }
    * { box-sizing:border-box; }
    html, body { overflow-x:clip; }
    body { margin:0; font-family:ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color:var(--ink); background:var(--paper); line-height:1.58; }
    main { max-width:1120px; margin:0 auto; padding:28px clamp(16px, 4vw, 48px) 64px; }
    .masthead { border-top:6px solid var(--accent); padding:24px 0 18px; display:grid; grid-template-columns:minmax(0, 1.35fr) minmax(280px, .65fr); gap:28px; align-items:start; }
    .eyebrow { margin:0 0 8px; color:var(--accent); text-transform:uppercase; letter-spacing:.08em; font-weight:850; font-size:12px; }
    h1 { margin:0; max-width:820px; font-size:clamp(30px, 4.4vw, 48px); line-height:1.05; letter-spacing:0; overflow-wrap:anywhere; }
    h2 { margin:0 0 12px; font-size:clamp(21px, 2.4vw, 28px); line-height:1.16; letter-spacing:0; }
    h3 { margin:18px 0 8px; }
    p { margin:0 0 14px; max-width:72ch; }
    .lede { color:var(--muted); font-size:18px; max-width:760px; margin-top:16px; }
    .chips { display:flex; flex-wrap:wrap; gap:8px; margin-top:18px; }
    .chips span { display:inline-flex; align-items:center; border:1px solid var(--line); border-radius:6px; padding:6px 9px; color:var(--muted); background:#fff; font-size:13px; }
    .decision-panel { min-width:0; border:1px solid var(--line); background:var(--panel); border-radius:8px; padding:18px; }
    .decision-panel h2 { font-size:18px; }
    .decision-panel p { color:var(--muted); }
    .spec-grid { margin:0; display:grid; grid-template-columns:repeat(2, minmax(0, 1fr)); gap:12px; }
    .spec-grid div { min-width:0; padding:10px 0; border-top:1px solid var(--line); }
    .spec-grid div:nth-child(-n+2) { border-top:0; }
    .spec-grid dt { margin:0 0 5px; color:var(--accent); font-size:12px; font-weight:850; text-transform:uppercase; letter-spacing:.06em; }
    .spec-grid dd { margin:0; color:var(--ink); }
    .doc-section, .review-gate { min-width:0; padding:22px 0; margin:0; border-top:1px solid var(--line); }
    .primary-section { padding:26px 0; }
    .evidence-section { color:#222; }
    .risk-section h2, .review-gate h2 { color:var(--risk); }
    ul { padding-left:22px; }
    li + li { margin-top:8px; }
    code { font-family:ui-monospace, SFMono-Regular, Menlo, monospace; border:1px solid #d8e3de; background:#f8fbf9; border-radius:5px; padding:2px 5px; overflow-wrap:anywhere; }
    pre { overflow:auto; padding:16px; border-radius:8px; background:#101827; color:#e9eefc; }
    pre code { border:0; padding:0; color:inherit; background:transparent; }
    .table-wrap { max-width:100%; overflow:auto; border:1px solid var(--line); border-radius:8px; margin:14px 0; }
    table { width:100%; border-collapse:collapse; min-width:720px; background:#fff; }
    th, td { border-bottom:1px solid var(--line); padding:10px 12px; text-align:left; vertical-align:top; }
    th { background:#eef3ef; }
    .muted { color:var(--muted); }
    .artifact-meta { margin-top:28px; padding-top:18px; border-top:1px solid var(--line); color:var(--muted); font-size:13px; }
    .artifact-meta dl { margin:0; display:grid; gap:8px; }
    .artifact-meta dt { font-weight:800; color:var(--ink); }
    .artifact-meta dd { margin:0 0 6px; }
    @media (max-width:900px) { .masthead, .spec-grid { grid-template-columns:1fr; } .spec-grid div:nth-child(2) { border-top:1px solid var(--line); } main { padding-top:18px; } }
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOptions {
    pub repo_root: PathBuf,
    pub source: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub output: PathBuf,
}

pub fn render(options: &RenderOptions) -> Result<RenderReport> {
    let markdown = fs::read_to_string(&options.source)
        .with_context(|| format!("failed to read {}", options.source.display()))?;
    let title = title_from(&markdown, &options.source);
    let packet_sections = sections(&markdown)?;
    let meta = metadata(&markdown)?;
    let missing: Vec<_> = REQUIRED_SECTIONS
        .iter()
        .filter(|name| !packet_sections.contains_key(**name))
        .copied()
        .collect();
    if !missing.is_empty() {
        bail!(
            "{}: missing required shaped-doc section(s): {}",
            options.source.display(),
            missing.join(", ")
        );
    }

    let source_rel = relative_or_self(&options.repo_root, &options.source);
    let output_rel = relative_or_self(&options.repo_root, &options.output);
    let source_hash = sha256(&options.source)?;
    let regenerate = format!(
        "cargo run --locked -p harness-kit-checks -- shape-render {} --output {}",
        source_rel.display(),
        output_rel.display()
    );
    let chips = meta
        .iter()
        .map(|(key, value)| {
            format!(
                "<span><b>{}</b> {}</span>",
                escape_html(&title_case(key)),
                escape_html(value)
            )
        })
        .collect::<String>();
    let mut body_sections = String::new();
    for name in BODY_SECTION_ORDER {
        if let Some(text) = packet_sections.get(*name) {
            body_sections.push_str(&section_markup(name, text)?);
        }
    }
    let deliverable = packet_sections
        .get("Deliverable")
        .map_or("", String::as_str);
    let goal = packet_sections.get("Goal").map_or("", String::as_str);
    let review_gate = r#"<section class="review-gate" id="browser-inspection">
    <h2>Review Gate</h2>
    <ul>
      <li>Open this artifact in a browser after generation.</li>
      <li>Confirm hierarchy, tables, implementation steps, and long code/path text fit on desktop and mobile widths.</li>
      <li>Record the inspected file path, viewport evidence, and residual visual risk in closeout.</li>
    </ul>
  </section>
"#;

    let html_doc = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="generator" content="harness-kit-checks shape-render">
  <meta name="harness-kit-shape-source" content="{source_rel}">
  <meta name="harness-kit-shape-source-sha256" content="{source_hash}">
  <meta name="harness-kit-shape-regenerate" content="{regenerate}">
  <title>{title} | Shape Packet</title>
  <style>{style}</style>
</head>
<body>
<main>
  <section class="masthead">
    <div>
      <p class="eyebrow">Operational PRD</p>
      <h1>{title}</h1>
      <p class="lede">{lede}</p>
      <div class="chips">{chips}<span><b>Source</b>{source_rel}</span></div>
    </div>
    <aside class="decision-panel">
      <h2>Deliverable</h2>
      {deliverable}
    </aside>
  </section>
  {body_sections}
  {review_gate}
  <section class="artifact-meta" aria-label="Artifact metadata">
    <dl>
      <div><dt>Source hash</dt><dd><code>{source_hash}</code></dd></div>
      <div><dt>Regenerate</dt><dd><code>{regenerate}</code></dd></div>
    </dl>
  </section>
</main>
</body>
</html>
"#,
        source_rel = escape_html(&source_rel.display().to_string()),
        source_hash = source_hash,
        regenerate = escape_html(&regenerate),
        title = escape_html(&title),
        style = STYLE,
        lede = markdown_inline(&first_paragraph(goal)),
        chips = chips,
        deliverable = key_value_grid(deliverable, Some(4))?,
        body_sections = body_sections,
        review_gate = review_gate,
    );

    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&options.output, html_doc)
        .with_context(|| format!("failed to write {}", options.output.display()))?;
    Ok(RenderReport {
        output: options.output.clone(),
    })
}

pub fn self_test() -> Result<()> {
    let fixture = r#"# Context Packet: Example

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
"#;
    let temp = tempfile::tempdir().context("failed to create temporary directory")?;
    let source = temp.path().join("packet.md");
    let output = temp.path().join("packet.html");
    fs::write(&source, fixture)?;
    render(&RenderOptions {
        repo_root: temp.path().to_path_buf(),
        source: source.clone(),
        output: output.clone(),
    })?;
    let text = fs::read_to_string(&output)?;
    for expected in [
        "Review Gate",
        "Implementation Sequence",
        "Deliverable",
        "Wrapped list text continues across lines.",
        "harness-kit-shape-source-sha256",
    ] {
        if !text.contains(expected) {
            bail!("missing rendered marker: {expected}");
        }
    }
    let bad_source = temp.path().join("bad.md");
    fs::write(
        &bad_source,
        "# Context Packet: Bad\n\n## Goal\n\nNo oracle.\n",
    )?;
    if render(&RenderOptions {
        repo_root: temp.path().to_path_buf(),
        source: bad_source,
        output: temp.path().join("bad.html"),
    })
    .is_ok()
    {
        bail!("renderer accepted incomplete packet");
    }
    Ok(())
}

fn slug(value: &str) -> String {
    let mut text = String::new();
    let mut previous_dash = false;
    for character in value.to_lowercase().chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-' {
            text.push(character);
            previous_dash = false;
        } else if !previous_dash {
            text.push('-');
            previous_dash = true;
        }
    }
    let text = text.trim_matches('-').to_string();
    if text.is_empty() {
        "context-packet".to_string()
    } else {
        text
    }
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn title_from(markdown: &str, source: &Path) -> String {
    markdown
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            source
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("context-packet")
                .to_string()
        })
}

fn metadata(markdown: &str) -> Result<Vec<(String, String)>> {
    let mut fields = Vec::new();
    for key in ["Priority", "Status", "Estimate"] {
        let regex = Regex::new(&format!(r"(?m)^{key}:\s*(.+)$"))?;
        if let Some(captures) = regex.captures(markdown) {
            fields.push((
                key.to_lowercase(),
                captures
                    .get(1)
                    .map_or("", |value| value.as_str())
                    .trim()
                    .to_string(),
            ));
        }
    }
    Ok(fields)
}

fn sections(markdown: &str) -> Result<std::collections::BTreeMap<String, String>> {
    let regex = Regex::new(r"(?m)^##\s+(.+?)\s*$")?;
    let matches: Vec<_> = regex.find_iter(markdown).collect();
    let mut result = std::collections::BTreeMap::new();
    for (index, found) in matches.iter().enumerate() {
        let line = found.as_str();
        let name = line.trim_start_matches('#').trim().to_string();
        let start = found.end();
        let end = matches
            .get(index + 1)
            .map_or(markdown.len(), regex::Match::start);
        result.insert(name, markdown[start..end].trim().to_string());
    }
    Ok(result)
}

fn first_paragraph(text: &str) -> String {
    text.split("\n\n")
        .map(str::trim)
        .find(|chunk| !chunk.is_empty())
        .unwrap_or("")
        .to_string()
}

fn key_values(text: &str) -> Result<Vec<(String, String)>> {
    let regex = Regex::new(r"^-\s+([^:]+):\s*(.+)$")?;
    let mut pairs = Vec::new();
    let mut current_key = String::new();
    let mut current_value: Vec<String> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some(captures) = regex.captures(line) {
            if !current_key.is_empty() {
                pairs.push((current_key, current_value.join(" ").trim().to_string()));
            }
            current_key = captures
                .get(1)
                .map_or("", |value| value.as_str())
                .trim()
                .to_string();
            current_value = vec![
                captures
                    .get(2)
                    .map_or("", |value| value.as_str())
                    .trim()
                    .to_string(),
            ];
        } else if !current_key.is_empty()
            && (raw.starts_with("  ") || raw.starts_with('\t'))
            && !line.trim().is_empty()
        {
            current_value.push(line.trim().to_string());
        } else if !current_key.is_empty() && line.trim().is_empty() {
            pairs.push((current_key, current_value.join(" ").trim().to_string()));
            current_key = String::new();
            current_value.clear();
        }
    }
    if !current_key.is_empty() {
        pairs.push((current_key, current_value.join(" ").trim().to_string()));
    }
    Ok(pairs)
}

fn key_value_grid(text: &str, limit: Option<usize>) -> Result<String> {
    let mut pairs = key_values(text)?;
    if let Some(limit) = limit {
        pairs.truncate(limit);
    }
    if pairs.is_empty() {
        return markdown_block(text);
    }
    let items = pairs
        .into_iter()
        .map(|(key, value)| {
            format!(
                "<div><dt>{}</dt><dd>{}</dd></div>",
                markdown_inline(&key),
                markdown_inline(&value)
            )
        })
        .collect::<String>();
    Ok(format!(r#"<dl class="spec-grid">{items}</dl>"#))
}

fn markdown_inline(value: &str) -> String {
    let escaped = escape_html(value);
    let code = Regex::new(r"`([^`]+)`").expect("valid regex");
    let strong = Regex::new(r"\*\*([^*]+)\*\*").expect("valid regex");
    let escaped = code.replace_all(&escaped, "<code>$1</code>");
    strong
        .replace_all(&escaped, "<strong>$1</strong>")
        .to_string()
}

fn markdown_block(text: &str) -> Result<String> {
    if text.is_empty() {
        return Ok(r#"<p class="muted">Not specified.</p>"#.to_string());
    }
    let ordered = Regex::new(r"^\d+\.\s+")?;
    let table_separator = Regex::new(r":?-{3,}:?")?;
    let mut output = Vec::new();
    let mut list_items: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut code_lines: Vec<String> = Vec::new();

    fn flush_list(output: &mut Vec<String>, list_items: &mut Vec<String>) {
        if !list_items.is_empty() {
            output.push(format!(
                "<ul>{}</ul>",
                list_items
                    .iter()
                    .map(|item| format!("<li>{item}</li>"))
                    .collect::<String>()
            ));
            list_items.clear();
        }
    }

    fn flush_paragraph(output: &mut Vec<String>, paragraph: &mut Vec<String>) {
        if !paragraph.is_empty() {
            output.push(format!("<p>{}</p>", markdown_inline(&paragraph.join(" "))));
            paragraph.clear();
        }
    }

    fn flush_table(
        output: &mut Vec<String>,
        table_rows: &mut Vec<Vec<String>>,
        table_separator: &Regex,
    ) {
        if table_rows.is_empty() {
            return;
        }
        let mut body = String::new();
        for (row_index, row) in table_rows.iter().enumerate() {
            if row_index == 1 && row.iter().all(|cell| table_separator.is_match(cell.trim())) {
                continue;
            }
            let tag = if row_index == 0 { "th" } else { "td" };
            body.push_str("<tr>");
            for cell in row {
                body.push_str(&format!("<{tag}>{}</{tag}>", markdown_inline(cell.trim())));
            }
            body.push_str("</tr>");
        }
        output.push(format!(
            r#"<div class="table-wrap"><table>{body}</table></div>"#
        ));
        table_rows.clear();
    }

    for raw in text.lines() {
        let line = raw.trim_end();
        if line.starts_with("```") {
            flush_paragraph(&mut output, &mut paragraph);
            flush_list(&mut output, &mut list_items);
            flush_table(&mut output, &mut table_rows, &table_separator);
            if in_code {
                output.push(format!(
                    "<pre><code>{}</code></pre>",
                    escape_html(&code_lines.join("\n"))
                ));
                code_lines.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            code_lines.push(line.to_string());
            continue;
        }
        if line.trim().is_empty() {
            flush_paragraph(&mut output, &mut paragraph);
            flush_list(&mut output, &mut list_items);
            flush_table(&mut output, &mut table_rows, &table_separator);
            continue;
        }
        if line.starts_with('|') && line.ends_with('|') {
            flush_paragraph(&mut output, &mut paragraph);
            flush_list(&mut output, &mut list_items);
            table_rows.push(
                line.trim_matches('|')
                    .split('|')
                    .map(ToString::to_string)
                    .collect(),
            );
            continue;
        }
        if (raw.starts_with("  ") || raw.starts_with('\t')) && !list_items.is_empty() {
            if let Some(last) = list_items.last_mut() {
                last.push(' ');
                last.push_str(&markdown_inline(line.trim()));
            }
            continue;
        }
        if line.starts_with("- ") || ordered.is_match(line) {
            flush_paragraph(&mut output, &mut paragraph);
            flush_table(&mut output, &mut table_rows, &table_separator);
            let item = if let Some(rest) = line.strip_prefix("- ") {
                rest.to_string()
            } else {
                ordered.replace(line, "").trim().to_string()
            };
            list_items.push(markdown_inline(item.trim()));
            continue;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            flush_paragraph(&mut output, &mut paragraph);
            flush_list(&mut output, &mut list_items);
            flush_table(&mut output, &mut table_rows, &table_separator);
            output.push(format!("<h3>{}</h3>", markdown_inline(heading.trim())));
            continue;
        }
        paragraph.push(line.trim().to_string());
    }
    flush_paragraph(&mut output, &mut paragraph);
    flush_list(&mut output, &mut list_items);
    flush_table(&mut output, &mut table_rows, &table_separator);
    if in_code {
        output.push(format!(
            "<pre><code>{}</code></pre>",
            escape_html(&code_lines.join("\n"))
        ));
    }
    Ok(output.join("\n"))
}

fn section_class(name: &str) -> &'static str {
    match name {
        "PRD Summary" | "Product Requirements" | "Technical Design" | "Deliverable" | "Oracle" => {
            "doc-section primary-section"
        }
        "Acceptance Evidence"
        | "Formal Spec"
        | "Delegation Evidence"
        | "Residual Risk"
        | "Observability Plan" => "doc-section evidence-section",
        _ if name.contains("Risk") => "doc-section risk-section",
        _ => "doc-section",
    }
}

fn section_markup(name: &str, text: &str) -> Result<String> {
    let body = if SPEC_GRID_SECTIONS.contains(&name) {
        key_value_grid(text, None)?
    } else {
        markdown_block(text)?
    };
    Ok(format!(
        r#"<section class="{}" id="{}"><h2>{}</h2>{}</section>"#,
        section_class(name),
        slug(name),
        escape_html(name),
        body
    ))
}

fn relative_or_self(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        self_test().unwrap();
    }

    #[test]
    fn missing_required_sections_reports_python_style_message() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("bad.md");
        fs::write(&source, "# Context Packet: Bad\n\n## Goal\n\nNo oracle.\n").unwrap();

        let error = render(&RenderOptions {
            repo_root: temp.path().to_path_buf(),
            source: source.clone(),
            output: temp.path().join("bad.html"),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains(&format!(
            "{}: missing required shaped-doc section(s): PRD Summary",
            source.display()
        )));
        assert!(error.contains("Risk + Rollout"));
    }

    #[test]
    fn markdown_block_preserves_core_markers() {
        let html = markdown_block(
            "A paragraph with `code` and **strong**.\n\n- Item\n  continuation\n\n| A | B |\n|---|---|\n| C | D |\n\n```rust\nfn main() {}\n```",
        )
        .unwrap();

        assert!(html.contains("<code>code</code>"));
        assert!(html.contains("<strong>strong</strong>"));
        assert!(html.contains("<li>Item continuation</li>"));
        assert!(html.contains("<table>"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn metadata_preserves_python_chip_order() {
        let meta = metadata("Estimate: M\nPriority: P1\nStatus: ready\n").unwrap();

        assert_eq!(
            meta,
            vec![
                ("priority".to_string(), "P1".to_string()),
                ("status".to_string(), "ready".to_string()),
                ("estimate".to_string(), "M".to_string()),
            ]
        );
    }
}
