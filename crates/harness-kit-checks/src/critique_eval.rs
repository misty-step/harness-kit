use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use regex::Regex;

pub fn grade(path: &Path) -> Result<String> {
    let text =
        fs::read_to_string(path).map_err(|error| anyhow::anyhow!("{}: {error}", path.display()))?;
    grade_text(&text)
}

pub fn grade_text(text: &str) -> Result<String> {
    require(text, r"Lens:.*ousterhout")?;
    require(text, r"Evidence:[[:space:]]+[^[:space:]]+:[0-9]+")?;
    require(text, r"Impact:")?;

    if matches(
        text,
        r"agents/ousterhout\.md|Ship|Conditional|Don't Ship|Dont Ship",
    ) {
        bail!("candidate turned targeted critique into static-agent or merge-verdict output");
    }

    Ok("PASS: critique output is lens-backed, evidence-backed, and non-verdict".to_string())
}

pub fn self_test() -> Result<String> {
    grade_text(
        "Lens: ousterhout\nFinding: shallow module\nEvidence: src/auth.rs:42\nImpact: hides coupling poorly\n",
    )?;
    expect_failure("Lens: ousterhout\nEvidence: src/auth.rs\nImpact: vague\n")?;
    expect_failure("Lens: ousterhout\nEvidence: src/auth.rs:42\nImpact: ok\nShip\n")?;
    Ok("PASS: critique eval self-test".to_string())
}

fn expect_failure(text: &str) -> Result<()> {
    match grade_text(text) {
        Ok(_) => bail!("expected critique eval fixture to fail"),
        Err(_) => Ok(()),
    }
}

fn require(text: &str, pattern: &str) -> Result<()> {
    if matches(text, pattern) {
        Ok(())
    } else {
        bail!("missing required pattern: {pattern}")
    }
}

fn matches(text: &str, pattern: &str) -> bool {
    Regex::new(&format!("(?i){pattern}"))
        .expect("critique eval regex must compile")
        .is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(self_test().unwrap(), "PASS: critique eval self-test");
    }

    #[test]
    fn rejects_static_agent_reference() {
        let error = grade_text(
            "Lens: ousterhout\nEvidence: src/auth.rs:42\nImpact: bad\nagents/ousterhout.md\n",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error,
            "candidate turned targeted critique into static-agent or merge-verdict output"
        );
    }
}
