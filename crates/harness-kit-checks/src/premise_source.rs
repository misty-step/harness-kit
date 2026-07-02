//! Restores the `/shape` premise-source verifier contract
//! (`skills/shape/SKILL.md`, backlog.d/113/095): a shape packet's
//! `## Premise Source` section must name a real, hash-pinned artifact or an
//! explicit waiver — never an unverified claim.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::lint_gates::GateReport;

const SECTION_HEADER: &str = "## Premise Source";
const WAIVER_PREFIX: &str = "Premise Source Waiver:";
const SOURCE_PREFIX: &str = "Premise Source:";

#[derive(Debug, PartialEq, Eq)]
enum PremiseClaim {
    Hashed { hash: String, reference: String },
    Waived { reason: String },
}

/// Real committed packets wrap the hash+reference onto the line after the
/// `Premise Source:` label and/or in a backtick code span (e.g.
/// `backlog.d/112-harness-eval-bench.md`) alongside the single-line form
/// SKILL.md documents (`backlog.d/_done/111-...md`). Tolerate both layouts;
/// stay strict on substance (hash shape, non-empty reference).
fn extract_claim(packet_text: &str) -> Result<PremiseClaim> {
    let section = section_body(packet_text).context("packet has no `## Premise Source` section")?;
    let lines: Vec<&str> = section
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    for (index, line) in lines.iter().enumerate() {
        if let Some(reason) = line.strip_prefix(WAIVER_PREFIX) {
            let reason = reason.trim();
            if reason.is_empty() {
                bail!("`{WAIVER_PREFIX}` line has no reason");
            }
            return Ok(PremiseClaim::Waived {
                reason: reason.to_string(),
            });
        }
        if let Some(rest) = line.strip_prefix(SOURCE_PREFIX) {
            let rest = rest.trim();
            let claim_line = if rest.is_empty() {
                lines.get(index + 1).copied().unwrap_or("")
            } else {
                rest
            };
            let claim_line = claim_line.trim_matches('`').trim();
            let mut parts = claim_line.splitn(2, char::is_whitespace);
            let hash_field = parts.next().unwrap_or("").trim();
            let reference = parts.next().unwrap_or("").trim();
            let hash = hash_field.strip_prefix("sha256:").with_context(|| {
                format!("Premise Source line missing `sha256:` prefix: {claim_line:?}")
            })?;
            if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                bail!("Premise Source hash is not 64 hex characters: {hash:?}");
            }
            if reference.is_empty() {
                bail!("Premise Source line has a hash but no artifact path or URL: {claim_line:?}");
            }
            return Ok(PremiseClaim::Hashed {
                hash: hash.to_string(),
                reference: reference.to_string(),
            });
        }
    }
    bail!(
        "packet's `## Premise Source` section has neither a `{SOURCE_PREFIX}` nor a `{WAIVER_PREFIX}` line"
    );
}

fn section_body(packet_text: &str) -> Option<&str> {
    let start = packet_text.find(SECTION_HEADER)?;
    let after_header = &packet_text[start + SECTION_HEADER.len()..];
    let end = after_header.find("\n## ").unwrap_or(after_header.len());
    Some(&after_header[..end])
}

fn is_url(reference: &str) -> bool {
    reference.contains("://")
}

/// CLI entry point: `premise-source validate PACKET [--repo PATH]|self-test`.
/// Kept here rather than in `main.rs`'s dispatch table so the growing CLI
/// surface doesn't push the already-grandfathered `main.rs` god-file further
/// past its ceiling — see backlog.d/113's notes.
pub fn run(args: &[String]) -> Result<GateReport> {
    let Some((subcommand, rest)) = args.split_first() else {
        bail!("usage: premise-source validate PACKET [--repo PATH]|self-test");
    };
    match subcommand.as_str() {
        "validate" => {
            let (packet, repo) = match rest {
                [packet] => (Path::new(packet), Path::new(".")),
                [packet, flag, repo] if flag == "--repo" => (Path::new(packet), Path::new(repo)),
                _ => bail!("usage: premise-source validate PACKET [--repo PATH]"),
            };
            validate_packet(packet, repo)
        }
        "self-test" => match rest {
            [] => self_test(),
            _ => bail!("usage: premise-source self-test"),
        },
        other => bail!("unknown premise-source subcommand: {other}"),
    }
}

/// Validates one shape packet's premise-source claim. Local artifact
/// references resolve relative to `repo_root`; URL references are accepted
/// structurally (their content cannot be hash-verified from here).
pub fn validate_packet(packet_path: &Path, repo_root: &Path) -> Result<GateReport> {
    let text = fs::read_to_string(packet_path)
        .with_context(|| format!("failed to read {}", packet_path.display()))?;
    validate_text(&text, repo_root).with_context(|| format!("{}", packet_path.display()))
}

fn validate_text(text: &str, repo_root: &Path) -> Result<GateReport> {
    match extract_claim(text)? {
        PremiseClaim::Waived { reason } => Ok(GateReport::success(format!(
            "premise source waived ({reason})"
        ))),
        PremiseClaim::Hashed { hash, reference } => {
            if is_url(&reference) {
                return Ok(GateReport::success(format!(
                    "premise source is a URL ({reference}) — hash unverifiable, structurally valid"
                )));
            }
            let artifact_path = repo_root.join(&reference);
            if !artifact_path.is_file() {
                bail!(
                    "premise source artifact does not exist: {}",
                    artifact_path.display()
                );
            }
            let bytes = fs::read(&artifact_path)
                .with_context(|| format!("failed to read {}", artifact_path.display()))?;
            let actual = format!("{:x}", Sha256::digest(&bytes));
            if actual != hash {
                bail!(
                    "premise source hash mismatch for {reference} — declared {hash}, actual {actual}"
                );
            }
            Ok(GateReport::success(format!(
                "premise source verified ({reference}, sha256 matches)"
            )))
        }
    }
}

struct Fixture {
    name: &'static str,
    packet: &'static str,
    artifact: Option<&'static [u8]>,
    should_pass: bool,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "missing premise source section fails",
        packet: "# Packet\n\n## Oracle\n\n- [ ] thing\n",
        artifact: None,
        should_pass: false,
    },
    Fixture {
        name: "missing local artifact fails",
        packet: "# Packet\n\n## Premise Source\n\nPremise Source: sha256:1111111111111111111111111111111111111111111111111111111111111111 does-not-exist.md\n\n## Oracle\n",
        artifact: None,
        should_pass: false,
    },
    Fixture {
        name: "hash mismatch fails",
        packet: "# Packet\n\n## Premise Source\n\nPremise Source: sha256:1111111111111111111111111111111111111111111111111111111111111111 premise.md\n\n## Oracle\n",
        artifact: Some(b"this is not the artifact that was hashed"),
        should_pass: false,
    },
    Fixture {
        name: "valid hash and local path passes",
        packet: "# Packet\n\n## Premise Source\n\nPremise Source: sha256:{{HASH}} premise.md\n\n## Oracle\n",
        artifact: Some(b"the real premise artifact content"),
        should_pass: true,
    },
    Fixture {
        name: "explicit waiver passes",
        packet: "# Packet\n\n## Premise Source\n\nPremise Source Waiver: no durable artifact exists; operator-prompt-derived.\n\n## Oracle\n",
        artifact: None,
        should_pass: true,
    },
    Fixture {
        name: "wrapped-line and backtick-quoted form passes (real committed shape, backlog.d/112)",
        packet: "# Packet\n\n## Premise Source\n\nPremise Source:\n`sha256:{{HASH}} premise.md`\n\n## Oracle\n",
        artifact: Some(b"the real premise artifact content"),
        should_pass: true,
    },
];

/// Runs the built-in fixture set and reports pass/fail per fixture — the
/// deterministic self-test SKILL.md names
/// (`premise-source self-test`), restoring 095's original oracle now that
/// the checker exists to run it against.
pub fn self_test() -> Result<GateReport> {
    let mut failures = Vec::new();
    for fixture in FIXTURES {
        let temp = tempfile::tempdir().context("failed to create fixture temp dir")?;
        let mut packet_text = fixture.packet.to_string();
        if let Some(bytes) = fixture.artifact {
            fs::write(temp.path().join("premise.md"), bytes)?;
            let hash = format!("{:x}", Sha256::digest(bytes));
            packet_text = packet_text.replace("{{HASH}}", &hash);
        }
        let result = validate_text(&packet_text, temp.path());
        let actually_passed = result.is_ok();
        if actually_passed != fixture.should_pass {
            failures.push(format!(
                "{}: expected {}, got {}{}",
                fixture.name,
                if fixture.should_pass { "PASS" } else { "FAIL" },
                if actually_passed { "PASS" } else { "FAIL" },
                result.err().map(|e| format!(" ({e})")).unwrap_or_default()
            ));
        }
    }
    if failures.is_empty() {
        Ok(GateReport::success(format!(
            "premise-source self-test: {} fixture(s) passed",
            FIXTURES.len()
        )))
    } else {
        bail!("premise-source self-test failed:\n{}", failures.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_fixtures_all_behave_as_declared() {
        self_test().expect("self_test should pass against its own fixtures");
    }

    #[test]
    fn validate_text_rejects_non_hex_hash() {
        let temp = tempfile::tempdir().unwrap();
        let packet =
            "## Premise Source\n\nPremise Source: sha256:not-a-hash premise.md\n\n## Oracle\n";
        let error = validate_text(packet, temp.path()).unwrap_err().to_string();
        assert!(error.contains("64 hex characters"));
    }

    #[test]
    fn validate_text_accepts_url_reference_without_hashing() {
        let temp = tempfile::tempdir().unwrap();
        let packet = "## Premise Source\n\nPremise Source: sha256:2222222222222222222222222222222222222222222222222222222222222222 https://example.com/issue/1\n\n## Oracle\n";
        let report = validate_text(packet, temp.path()).unwrap();
        assert!(report.ok_message.contains("URL"));
    }

    #[test]
    fn validate_text_rejects_empty_waiver_reason() {
        let temp = tempfile::tempdir().unwrap();
        let packet = "## Premise Source\n\nPremise Source Waiver:\n\n## Oracle\n";
        let error = validate_text(packet, temp.path()).unwrap_err().to_string();
        assert!(error.contains("no reason"));
    }

    #[test]
    fn validate_packet_reads_from_disk() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("premise.md"), b"content").unwrap();
        let hash = format!("{:x}", Sha256::digest(b"content"));
        let packet_path = temp.path().join("packet.md");
        fs::write(
            &packet_path,
            format!("## Premise Source\n\nPremise Source: sha256:{hash} premise.md\n\n## Oracle\n"),
        )
        .unwrap();

        let report = validate_packet(&packet_path, temp.path()).unwrap();
        assert!(report.ok_message.contains("verified"));
    }

    #[test]
    fn validate_packet_wraps_error_with_packet_path() {
        let temp = tempfile::tempdir().unwrap();
        let packet_path = temp.path().join("empty.md");
        fs::write(&packet_path, "# Packet\n\n## Oracle\n").unwrap();

        let error = validate_packet(&packet_path, temp.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("empty.md"));
    }

    #[test]
    fn run_dispatches_self_test() {
        let report = run(&["self-test".to_string()]).unwrap();
        assert!(report.ok_message.contains("fixture"));
    }

    #[test]
    fn run_dispatches_validate_with_and_without_repo_flag() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("premise.md"), b"content").unwrap();
        let hash = format!("{:x}", Sha256::digest(b"content"));
        let packet_path = temp.path().join("packet.md");
        fs::write(
            &packet_path,
            format!("## Premise Source\n\nPremise Source: sha256:{hash} premise.md\n\n## Oracle\n"),
        )
        .unwrap();

        let report = run(&[
            "validate".to_string(),
            packet_path.to_string_lossy().to_string(),
            "--repo".to_string(),
            temp.path().to_string_lossy().to_string(),
        ])
        .unwrap();
        assert!(report.ok_message.contains("verified"));
    }

    #[test]
    fn run_rejects_unknown_subcommand_and_missing_args() {
        assert!(run(&[]).is_err());
        assert!(run(&["bogus".to_string()]).is_err());
        assert!(run(&["validate".to_string()]).is_err());
        assert!(run(&["self-test".to_string(), "extra".to_string()]).is_err());
    }
}
