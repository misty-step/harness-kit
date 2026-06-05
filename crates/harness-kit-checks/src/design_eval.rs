use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignEvalMode {
    RenderedCritique,
    ScaffoldContract,
    DesignContractMaintenance,
    TokenOnlyCritique,
}

impl DesignEvalMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "rendered-critique" => Some(Self::RenderedCritique),
            "scaffold-contract" => Some(Self::ScaffoldContract),
            "design-contract-maintenance" => Some(Self::DesignContractMaintenance),
            "token-only-critique" => Some(Self::TokenOnlyCritique),
            _ => None,
        }
    }
}

pub fn grade(mode: DesignEvalMode, output_path: &Path) -> Result<String> {
    let text = fs::read_to_string(output_path)
        .map_err(|error| anyhow::anyhow!("{}: {error}", output_path.display()))?;
    grade_text(mode, &text)
}

pub fn grade_text(mode: DesignEvalMode, text: &str) -> Result<String> {
    match mode {
        DesignEvalMode::RenderedCritique => grade_rendered_critique(text),
        DesignEvalMode::ScaffoldContract => grade_scaffold_contract(text),
        DesignEvalMode::DesignContractMaintenance => grade_design_contract_maintenance(text),
        DesignEvalMode::TokenOnlyCritique => grade_token_only_critique(text),
    }
}

pub fn self_test() -> Result<String> {
    let temp = tempfile::tempdir()?;
    let scaffold_pass = temp.path().join("scaffold-pass.md");
    let scaffold_fail = temp.path().join("scaffold-fail.md");
    let token_pass = temp.path().join("token-pass.md");
    let token_fail = temp.path().join("token-fail.md");
    let maintenance_pass = temp.path().join("maintenance-pass.md");
    let maintenance_fail = temp.path().join("maintenance-fail.md");

    fs::write(
        &scaffold_pass,
        r#"Generated files:

- DESIGN.md
- design-contract.md

DESIGN.md sections:

1. Product Intent
2. Audience and Context
3. Brand Attributes
4. Visual Language
5. Layout and Density
6. Components and Interaction
7. Content Voice
8. Accessibility and Responsiveness
9. Evidence and Governance

design-contract.md:

| Source | Fact | Provenance | Confidence | Use | Evidence / Notes |
|---|---|---|---|---|---|
| app screenshot | Dense operator dashboard | observed | high | keep | Rendered dashboard artifact |
| user note | Avoid playful illustration | provided | medium | change | Stakeholder brief |
| competitor site | Hero motion direction | inferred | low | do-not-copy | Reference brand only |
"#,
    )?;
    fs::write(
        &scaffold_fail,
        r#"The app brand is premium fintech with cinematic depth.

Create DESIGN.md with these sections:

1. Product Intent
2. Audience and Context
3. Brand Attributes
4. Visual Language
5. Layout and Density
6. Components and Interaction
7. Content Voice
8. Accessibility and Responsiveness
9. Evidence and Governance

Use the competitor reference directly.
"#,
    )?;
    fs::write(
        &token_pass,
        r#"Token file inspected: src/theme.ts.

Unverified caveat: I cannot make a final design judgment because no screenshot,
rendered route, URL, or artifact was available. The token layer suggests a
coherent spacing scale, but rendered evidence is still required.
"#,
    )?;
    fs::write(
        &token_fail,
        r#"The token document has colors, spacing, and component names. The design is
complete and ready to ship.
"#,
    )?;
    fs::write(
        &maintenance_pass,
        r#"Created DESIGN.md and updated design-contract.md.

Facts are labeled observed, provided, and inferred. The competitor reference is
marked do-not-copy.
"#,
    )?;
    fs::write(
        &maintenance_fail,
        r#"Skipped DESIGN.md because this change touches durable product-facing visual language.
No contract update needed.
"#,
    )?;

    grade(DesignEvalMode::ScaffoldContract, &scaffold_pass)?;
    expect_failure(
        DesignEvalMode::ScaffoldContract,
        &scaffold_fail,
        "expected scaffold output without provenance/do-not-copy to fail",
    )?;
    grade(DesignEvalMode::TokenOnlyCritique, &token_pass)?;
    expect_failure(
        DesignEvalMode::TokenOnlyCritique,
        &token_fail,
        "expected token-only success claim to fail",
    )?;
    grade(DesignEvalMode::DesignContractMaintenance, &maintenance_pass)?;
    expect_failure(
        DesignEvalMode::DesignContractMaintenance,
        &maintenance_fail,
        "expected missing design contract maintenance output to fail",
    )?;

    Ok("PASS: design eval self-test".to_string())
}

fn grade_rendered_critique(text: &str) -> Result<String> {
    require_any(text, "screenshot|rendered|artifact")?;
    require_any(text, "operational|on-call|operator|workbench")?;
    require_any(text, "hierarchy")?;
    require_any(text, "density|spacing")?;
    require_any(text, "typograph|heading|font")?;
    require_any(text, "focus state|focus ring|keyboard|icon-only|a11y")?;
    require_any(text, "Design Gate")?;

    if matches_any(
        text,
        "install (a )?(framework|component library)|add (a )?(framework|component library)|new token engine|global token",
    ) {
        bail!("candidate over-scoped one-off design critique into framework/token work");
    }

    Ok("PASS: design output critiques rendered dashboard without framework drift".to_string())
}

fn grade_scaffold_contract(text: &str) -> Result<String> {
    require_any(text, "DESIGN\\.md")?;
    require_any(text, "design-contract\\.md")?;
    for section in [
        "Product Intent",
        "Audience and Context",
        "Brand Attributes",
        "Visual Language",
        "Layout and Density",
        "Components and Interaction",
        "Content Voice",
        "Accessibility and Responsiveness",
        "Evidence and Governance",
    ] {
        require_any(text, section)?;
    }
    require_any(text, "Source.*Fact.*Provenance.*Confidence.*Use")?;
    for term in ["observed", "provided", "inferred", "keep", "change"] {
        require_any(text, term)?;
    }
    require_any(text, "do-not-copy|do not copy")?;

    if matches_any(text, "brand is|brand should be|visual language is")
        && !matches_any(text, "observed|provided|inferred")
    {
        bail!("candidate invents design facts without provenance labels");
    }

    Ok("PASS: design scaffold records repo-owned design facts with provenance".to_string())
}

fn grade_design_contract_maintenance(text: &str) -> Result<String> {
    require_any(text, "DESIGN\\.md")?;
    require_any(text, "read|created|updated|maintained")?;
    require_any(text, "design-contract\\.md")?;
    require_any(text, "observed|provided|inferred")?;
    require_any(text, "do-not-copy|do not copy")?;

    if matches_any(text, "one-off|internal|no durable") {
        require_any(text, "waiver|not applicable|no-durable-fact")?;
    }

    if matches_any(
        text,
        "DESIGN\\.md.*not (needed|required|applicable)|skip(ped)? DESIGN\\.md",
    ) && !matches_any(
        text,
        "one-off|internal|no durable|waiver|not applicable|no-durable-fact",
    ) {
        bail!("candidate skips DESIGN.md without a one-off/internal/no-durable-fact waiver");
    }

    Ok("PASS: design output maintains or waives DESIGN.md contract with provenance".to_string())
}

fn grade_token_only_critique(text: &str) -> Result<String> {
    if !matches_any(
        text,
        "screenshot|rendered|artifact|URL|route|unverified|cannot make a final design judgment|rendering is impossible",
    ) {
        bail!("candidate lacks rendered evidence or explicit unverified caveat");
    }

    if matches_any(
        text,
        "ready to ship|design succeeds|design passes|design is complete",
    ) && !matches_any(
        text,
        "screenshot|rendered|artifact|unverified|cannot make a final design judgment|rendering is impossible",
    ) {
        bail!("candidate claims design success from tokens/docs alone");
    }

    Ok("PASS: token-only critique preserves rendered-evidence caveat".to_string())
}

fn expect_failure(mode: DesignEvalMode, path: &Path, message: &str) -> Result<()> {
    match grade(mode, path) {
        Ok(_) => bail!("{message}"),
        Err(_) => Ok(()),
    }
}

fn require_any(text: &str, pattern: &str) -> Result<()> {
    if matches_any(text, pattern) {
        Ok(())
    } else {
        bail!("missing required pattern: {pattern}")
    }
}

fn matches_any(text: &str, pattern: &str) -> bool {
    Regex::new(&format!("(?i){pattern}"))
        .expect("design eval regex must compile")
        .is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(self_test().unwrap(), "PASS: design eval self-test");
    }

    #[test]
    fn rendered_critique_rejects_framework_drift() {
        let error = grade_text(
            DesignEvalMode::RenderedCritique,
            "Screenshot artifact of operator workbench. Hierarchy, density, typography, keyboard focus state. Design Gate. Add a framework.",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error,
            "candidate over-scoped one-off design critique into framework/token work"
        );
    }

    #[test]
    fn unknown_mode_is_rejected() {
        assert_eq!(DesignEvalMode::parse("missing"), None);
    }
}
