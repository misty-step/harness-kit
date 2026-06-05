use anyhow::{Result, bail};
use regex::Regex;

pub const MARKER_START: &str = "<!-- harness-kit-agent-transcript:start -->";
pub const MARKER_END: &str = "<!-- harness-kit-agent-transcript:end -->";

pub fn redact(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let kept = normalized
        .lines()
        .filter(|line| !drop_line(line))
        .collect::<Vec<_>>()
        .join("\n");
    let replacements = [
        (
            r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
            "[REDACTED_PRIVATE_KEY]",
        ),
        (
            r"\b(?:sk|rk|ghp|glpat|xox[abprs]?)-[A-Za-z0-9_\-]{16,}\b",
            "[REDACTED_TOKEN]",
        ),
        (r"\bgithub_pat_[A-Za-z0-9_]{20,}\b", "[REDACTED_GITHUB_PAT]"),
        (r"\bAKIA[0-9A-Z]{16}\b", "[REDACTED_AWS_KEY]"),
        (
            r#"(?i)\b(cookie|authorization|x-api-key|api[_-]?key|token|password)\s*[:=]\s*[^\s`'"]{8,}"#,
            "$1=[REDACTED_SECRET]",
        ),
        (
            r#"https?://[^\s)>"]*(?:code|token|access_token|refresh_token)=[^\s)>"]+"#,
            "[REDACTED_AUTH_URL]",
        ),
        (r"/Users/[A-Za-z0-9_.-]+/", "~/"),
    ];
    let mut output = kept;
    for (pattern, replacement) in replacements {
        output = Regex::new(pattern)
            .unwrap()
            .replace_all(&output, replacement)
            .to_string();
    }
    output.trim().to_string()
}

pub fn assert_safe(text: &str) -> Result<()> {
    for pattern in [
        r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
        r"\b(?:sk|rk|ghp|glpat|xox[abprs]?)-[A-Za-z0-9_\-]{16,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{20,}\b",
        r"\bAKIA[0-9A-Z]{16}\b",
    ] {
        if Regex::new(pattern).unwrap().is_match(text) {
            bail!("unsafe transcript: unresolved secret pattern {pattern:?}");
        }
    }
    let assignment_pattern = r#"(?i)\b(cookie|authorization|x-api-key|api[_-]?key|token|password)\s*[:=]\s*[^\s`'"]{8,}"#;
    let assignment = Regex::new(assignment_pattern).unwrap();
    for matched in assignment.find_iter(text) {
        if !matched.as_str().contains("[REDACTED") {
            bail!("unsafe transcript: unresolved secret pattern {assignment_pattern:?}");
        }
    }
    Ok(())
}

pub fn render_block(text: &str, title: &str) -> Result<String> {
    let safe = redact(text);
    assert_safe(&safe)?;
    Ok(format!(
        "{MARKER_START}\n<details>\n<summary>{}</summary>\n\n```text\n{}\n```\n\n</details>\n{MARKER_END}\n",
        escape_html(title),
        escape_html_no_quotes(&safe)
    ))
}

pub fn self_test() -> Result<&'static str> {
    let sample = r#"user: fix the bug
assistant: running tests
Authorization: Bearer sk-test_1234567890abcdef
path: /Users/alice/project/file.py
-----BEGIN PRIVATE KEY-----
abc
-----END PRIVATE KEY-----
tool: pytest passed
"#;
    let rendered = render_block(sample, "Self Test")?;
    assert!(rendered.contains("[REDACTED_TOKEN]"));
    assert!(rendered.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(rendered.contains("~/project/file.py"));
    assert!(!rendered.contains("sk-test"));
    Ok("agent-transcript self-test ok")
}

fn drop_line(line: &str) -> bool {
    Regex::new(r"(?i)^\s*(system|developer)\b")
        .unwrap()
        .is_match(line)
        || Regex::new(r"(?i)\b(raw tool output|environment dump|export -p|^env$|^set$)\b")
            .unwrap()
            .is_match(line)
}

fn escape_html(text: &str) -> String {
    escape_html_no_quotes(text)
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn escape_html_no_quotes(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_redacted_collapsible_block() {
        let sample = r#"system do not include this
user: fix <bug>
Authorization: Bearer sk-test_1234567890abcdef
path: /Users/alice/project/file.py
-----BEGIN PRIVATE KEY-----
abc
-----END PRIVATE KEY-----
"#;

        let rendered = render_block(sample, "My <Transcript>").unwrap();

        assert!(rendered.starts_with(MARKER_START));
        assert!(rendered.contains("<summary>My &lt;Transcript&gt;</summary>"));
        assert!(rendered.contains("user: fix &lt;bug&gt;"));
        assert!(rendered.contains("[REDACTED_TOKEN]"));
        assert!(rendered.contains("[REDACTED_PRIVATE_KEY]"));
        assert!(rendered.contains("~/project/file.py"));
        assert!(!rendered.contains("system do not include"));
        assert!(!rendered.contains("sk-test"));
    }

    #[test]
    fn blocks_unresolved_secret_after_redaction() {
        let error = assert_safe("password=unredacted").unwrap_err();

        assert!(error.to_string().contains("unsafe transcript"));
    }

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(self_test().unwrap(), "agent-transcript self-test ok");
    }
}
