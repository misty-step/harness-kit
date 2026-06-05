use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceBlock {
    pub path: PathBuf,
    pub heading: String,
    pub line: usize,
    pub fields: HashMap<String, (String, usize)>,
}

const BLOCK_HEADINGS: [&str; 3] = ["Completion Gate", "Acceptance Evidence", "Formal Spec"];
const PLACEHOLDER_VALUES: [&str; 10] = [
    "",
    "???",
    "fixme",
    "n/a",
    "na",
    "none",
    "placeholder",
    "tbd",
    "todo",
    "unknown",
];

pub fn check_paths(paths: &[PathBuf]) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    for root in paths {
        for path in markdown_files(root)? {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("{}: cannot read markdown as text", path.display()))?;
            for block in parse_evidence_blocks(&path, &text) {
                errors.extend(check_block(&block));
            }
        }
    }
    Ok(errors)
}

pub fn parse_evidence_blocks(path: &Path, text: &str) -> Vec<EvidenceBlock> {
    let lines: Vec<&str> = text.lines().collect();
    let field_re = Regex::new(r"^\s*-\s+([^:\n]+):\s*(.*)$").expect("valid field regex");
    let heading_re = Regex::new(r"^(#{2,3})\s+(.+?)\s*$").expect("valid heading regex");

    let mut blocks = Vec::new();
    let mut index = 0;
    let mut in_fence = false;

    while index < lines.len() {
        let stripped = lines[index].trim();
        if is_fence(stripped) {
            in_fence = !in_fence;
            index += 1;
            continue;
        }

        if let Some(captures) = heading_re.captures(stripped) {
            let heading = captures
                .get(2)
                .map(|heading| heading.as_str())
                .unwrap_or("");
            if in_fence || !BLOCK_HEADINGS.contains(&heading) {
                index += 1;
                continue;
            }
            let start_line = index + 1;
            let start_level = captures.get(1).expect("heading level").as_str().len();
            let mut fields = HashMap::new();
            let mut cursor = index + 1;
            let mut section_in_fence = in_fence;

            while cursor < lines.len() {
                let current = lines[cursor].trim();
                if is_fence(current) {
                    section_in_fence = !section_in_fence;
                    cursor += 1;
                    continue;
                }

                let current_heading = heading_re.captures(current);
                if !section_in_fence
                    && cursor != index
                    && let Some(current_heading) = current_heading.as_ref()
                    && current_heading
                        .get(1)
                        .expect("heading level")
                        .as_str()
                        .len()
                        <= start_level
                {
                    break;
                }

                if let Some(field) = field_re.captures(lines[cursor]) {
                    let name = field
                        .get(1)
                        .expect("field name")
                        .as_str()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
                    let value = field
                        .get(2)
                        .expect("field value")
                        .as_str()
                        .trim()
                        .to_string();
                    fields.insert(name, (value, cursor + 1));
                }
                cursor += 1;
            }

            blocks.push(EvidenceBlock {
                path: path.to_path_buf(),
                heading: heading.to_string(),
                line: start_line,
                fields,
            });
            index = cursor;
            in_fence = section_in_fence;
            continue;
        }

        index += 1;
    }

    blocks
}

pub fn check_block(block: &EvidenceBlock) -> Vec<String> {
    let mut errors = Vec::new();
    for field in required_fields(&block.heading) {
        if !block.fields.contains_key(*field) {
            errors.push(format!(
                "{}:{}: {} missing field {:?}",
                block.path.display(),
                block.line,
                block.heading,
                field
            ));
        }
    }

    let mut fields: Vec<_> = block.fields.iter().collect();
    fields.sort_by_key(|(_, (_, line))| *line);
    for (field, (value, line)) in fields {
        if is_placeholder(value) {
            errors.push(format!(
                "{}:{}: {} field {:?} has blank or placeholder-only evidence",
                block.path.display(),
                line,
                block.heading,
                field
            ));
        }
    }

    errors
}

fn required_fields(heading: &str) -> &'static [&'static str] {
    match heading {
        "Completion Gate" => &[
            "Evidence that proves it",
            "Exact command/path/route exercised",
            "Residual risk",
        ],
        "Acceptance Evidence" => &[
            "Acceptance source",
            "Evidence that proves it",
            "Exact command/path/route exercised",
            "Oracle / acceptance artifact hash",
            "Contract-change acknowledgment",
            "Residual risk",
        ],
        "Formal Spec" => &[
            "Formal Spec Required",
            "Informal spec",
            "Formal examples",
            "Acceptance oracle",
            "Hardening budget",
            "Waiver path",
        ],
        _ => &[],
    }
}

fn is_placeholder(value: &str) -> bool {
    let normalized = value.trim().trim_end_matches('.').to_lowercase();
    if PLACEHOLDER_VALUES.contains(&normalized.as_str()) {
        return true;
    }
    let trimmed = value.trim();
    (trimmed.starts_with('<') && trimmed.ends_with('>'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn is_fence(line: &str) -> bool {
    line.starts_with("```")
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut paths = Vec::new();
    collect_markdown_files(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_markdown_files(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path
            .components()
            .any(|component| component.as_os_str() == ".external")
        {
            continue;
        }
        if path.is_dir() {
            collect_markdown_files(&path, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn _headings_set() -> HashSet<&'static str> {
    BLOCK_HEADINGS.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_completion_gate_accepts_required_fields() {
        let text = "\
## Completion Gate
- Evidence that proves it: focused test fails before the fix and passes after.
- Exact command/path/route exercised: pytest tests/test_login.py -q.
- Residual risk: OAuth provider outage path remains unverified.
";

        let blocks = parse_evidence_blocks(Path::new("fixture.md"), text);
        assert_eq!(check_block(&blocks[0]), Vec::<String>::new());
    }

    #[test]
    fn completion_gate_rejects_missing_required_field() {
        let text = "\
## Completion Gate
- Evidence that proves it: smoke test output.
- Residual risk: none beyond provider outage.
";

        let block = parse_evidence_blocks(Path::new("fixture.md"), text)
            .into_iter()
            .next()
            .unwrap();
        let errors = check_block(&block);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("Exact command/path/route exercised"))
        );
    }

    #[test]
    fn completion_gate_rejects_blank_and_placeholder_values() {
        let text = "\
## Completion Gate
- Evidence that proves it: TBD
- Exact command/path/route exercised:
- Residual risk: <unknown>
";

        let block = parse_evidence_blocks(Path::new("fixture.md"), text)
            .into_iter()
            .next()
            .unwrap();
        let errors = check_block(&block);
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().all(|error| error.contains("placeholder")));
    }

    #[test]
    fn valid_acceptance_evidence_accepts_contract_fields() {
        let text = "\
## Acceptance Evidence
- Acceptance source: docs/spec.md plus fixtures/auth.json.
- Evidence that proves it: mutated fixture failed the acceptance path.
- Exact command/path/route exercised: npm run test:e2e -- auth.
- Oracle / acceptance artifact hash: sha256:abc123 for fixtures/auth.json.
- Contract-change acknowledgment: no acceptance contract changed.
- Residual risk: browser-specific layout remains out of scope.
";

        let block = parse_evidence_blocks(Path::new("fixture.md"), text)
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(check_block(&block), Vec::<String>::new());
    }

    #[test]
    fn outer_heading_collects_inner_fenced_template_once() {
        let text = "\
## Completion Gate

Every report includes:

```markdown
## Completion Gate
- Evidence that proves it: command output copied into the brief.
- Exact command/path/route exercised: dagger call check --source=.
- Residual risk: none known.
```

## Gotchas
";

        let blocks = parse_evidence_blocks(Path::new("fixture.md"), text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(check_block(&blocks[0]), Vec::<String>::new());
    }

    #[test]
    fn h3_completion_gate_terminates_at_next_h3() {
        let text = "\
## Report

### Completion Gate
- Evidence that proves it: command output copied into the report.
- Exact command/path/route exercised: python3 scripts/check.py.
- Residual risk: none known.

### Residual Risks
- Follow-up:
";

        let blocks = parse_evidence_blocks(Path::new("fixture.md"), text);
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].fields.contains_key("Follow-up"));
        assert_eq!(check_block(&blocks[0]), Vec::<String>::new());
    }
}
