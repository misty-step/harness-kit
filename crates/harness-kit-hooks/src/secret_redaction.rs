//! Secret-shape redaction for Bash command output (harness-kit-915).
//!
//! Ports and extends the masking core proven in
//! `doomscrum/src/secrets.rs` (independently built prior art, cited in the
//! misty-step-928 research briefing) rather than hand-rolling detection from
//! zero. Extends its `SECRET_PREFIXES` with the additional shapes the card
//! names explicitly: `AKIA` (AWS), `xoxb-`/`xoxp-` (Slack), JWTs, and PEM
//! private-key headers.
//!
//! ARCHITECTURE NOTE, corrected from the card's literal text after checking
//! the live Claude Code hooks docs: a PostToolUse hook's `updatedToolOutput`
//! only changes what the model sees in-context for the rest of the
//! conversation — by the time PostToolUse fires, the raw tool result
//! (including any secret) is already captured into the transcript JSONL and
//! OTel telemetry. To keep a secret out of the transcript at all, the
//! rewrite has to happen at PreToolUse, before the command's real stdout/
//! stderr are ever captured: rewrite the Bash `command` to pipe its own
//! output through this redactor via process substitution
//! (`> >(redact) 2> >(redact >&2)`), so Claude Code captures the
//! already-redacted bytes as the tool result. See
//! `claude_hooks::secret_redaction_command_rewrite`.
//!
//! `gitleaks stdin` (already installed, the actual upstream tool the card's
//! named LLM-Redactor project itself is Gitleaks-*compatible* with) was
//! evaluated as a substitute for hand-rolled detection. Live-tested: it
//! reliably catches high-entropy generic secrets (e.g. `sk-...` shaped
//! tokens) but its *default* ruleset did NOT fire on synthetic
//! `ghp_`/`AKIA`/`xoxb-`-shaped strings in isolated stdout text (its rules
//! for those services expect more surrounding context than a bare command
//! output line provides). Conclusion: gitleaks is real, valuable defense in
//! depth for the generic/high-entropy class, not a complete substitute for
//! prefix/shape rules on the specific named services — so both run.

use std::collections::HashSet;

/// Secret prefixes/shapes to mask regardless of source. Extends doomscrum's
/// `SECRET_PREFIXES` (`sk-`, GitHub tokens) with the shapes harness-kit-915
/// names explicitly: AWS access keys, Slack bot/user tokens, and PEM
/// private-key headers. JWTs are handled separately (three dot-separated
/// base64url segments), not as a fixed prefix.
const SECRET_PREFIXES: &[&str] = &[
    "sk-", "ghp_", "gho_", "ghs_", "ghu_", "ghr_", "github_pat_", "AKIA", "ASIA", "xoxb-", "xoxp-", "xoxa-", "xoxr-",
];

const PEM_HEADERS: &[&str] = &[
    "-----BEGIN PRIVATE KEY-----",
    "-----BEGIN RSA PRIVATE KEY-----",
    "-----BEGIN EC PRIVATE KEY-----",
    "-----BEGIN OPENSSH PRIVATE KEY-----",
    "-----BEGIN PGP PRIVATE KEY BLOCK-----",
];

const REDACTED: &str = "[REDACTED]";

/// Mask credential-shaped tokens in `text`. `extra` holds literal secret
/// values to mask by exact match first (kept as a parameter, same as
/// doomscrum's `redact`, so the core stays pure/testable without touching
/// the environment or a live 1Password call).
pub fn redact(text: &str, extra: &[String]) -> String {
    let mut s = text.to_string();
    for v in extra {
        if v.len() >= 6 {
            s = s.replace(v.as_str(), REDACTED);
        }
    }
    let s = redact_pem_blocks(&s);
    redact_tokens(&s)
}

/// PEM blocks span multiple lines (`-----BEGIN ... -----` through
/// `-----END ... -----`); mask the whole block since the header alone
/// doesn't carry the secret, but a truncated/no-footer block still must not
/// leak key material that follows the header.
fn redact_pem_blocks(text: &str) -> String {
    let Some(begin_idx) = PEM_HEADERS
        .iter()
        .filter_map(|header| text.find(header))
        .min()
    else {
        return text.to_string();
    };
    let after_begin = &text[begin_idx..];
    let end_marker = "-----END ";
    match after_begin.find(end_marker) {
        Some(end_rel) => {
            let tail_after_end = &after_begin[end_rel..];
            let line_end = tail_after_end.find('\n').unwrap_or(tail_after_end.len());
            let block_end = begin_idx + end_rel + line_end;
            format!("{}{REDACTED}{}", &text[..begin_idx], &text[block_end..])
        }
        None => format!("{}{REDACTED}", &text[..begin_idx]),
    }
}

fn redact_tokens(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut token = String::new();
    let mut prev = String::new();
    // Tracks whether this token is a shell-variable reference (`$VAR` or
    // `${VAR}`) rather than a literal value -- both `$` immediately before
    // the token and `${` (dollar then an opening brace, with nothing
    // alphanumeric in between) count.
    let mut dollar_ref = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '+' | '/' | '=' | '.' | '~' | ':') {
            token.push(ch);
            continue;
        }
        if !token.is_empty() {
            emit(&mut out, &token, &prev, dollar_ref);
            prev = std::mem::take(&mut token);
        }
        dollar_ref = ch == '$' || (ch == '{' && dollar_ref);
        out.push(ch);
    }
    if !token.is_empty() {
        emit(&mut out, &token, &prev, dollar_ref);
    }
    out
}

/// Emits `token`, redacting from the first secret-prefix match onward.
///
/// Live-verified bug this fixes: a naive per-segment `starts_with(prefix)`
/// check (splitting only on `/:@=`) misses a secret glued to arbitrary text
/// via a hyphen, e.g. `err-side-sk-or-v1-XYZ...` — hyphen is a token-
/// continuation character (needed so `sk-or-v1-...` itself stays one
/// token), so the whole glued string is one segment that never *starts
/// with* `sk-`. Searching for the prefix anywhere in the token, then
/// redacting from that position to the token's end, catches this while a
/// bare word like `pre-existing` still survives (no known prefix appears in
/// it at all).
fn emit(out: &mut String, token: &str, prev: &str, dollar_ref: bool) {
    // A `Bearer $VARNAME` shell-variable reference (as opposed to a resolved
    // value) is not a leak -- it's a pointer to one, definitionally absent
    // from the text. Found via the harness-kit-915 transcript scan: this
    // shape (curl commands quoting `-H "Authorization: Bearer $TOKEN"` or
    // `${TOKEN}`) dominated the false-positive count in a manual sample of
    // the real corpus.
    if prev.eq_ignore_ascii_case("bearer") && token.len() >= 8 && !dollar_ref {
        out.push_str(REDACTED);
        return;
    }
    if looks_like_jwt(token) || token.split(['/', '@', '=']).any(looks_like_fal_key) {
        out.push_str(REDACTED);
        return;
    }
    if let Some(start) = find_secret_prefix_start(token) {
        out.push_str(&token[..start]);
        out.push_str(REDACTED);
        return;
    }
    out.push_str(token);
}

/// Byte offset of the earliest secret-prefix match whose material AFTER the
/// prefix (up to the next `/:@=` delimiter) is at least 8 bytes long — the
/// length floor counts only what follows the prefix, not the prefix itself,
/// so a bare prefix word in prose (`sk-users`, `the sk- prefix`) survives
/// while real key material (`sk-or-v1-ABCDEF1234567890`) is caught.
fn find_secret_prefix_start(token: &str) -> Option<usize> {
    SECRET_PREFIXES
        .iter()
        .filter_map(|prefix| {
            let idx = token.find(prefix)?;
            let after_prefix = &token[idx + prefix.len()..];
            let material_len = after_prefix.find(['/', ':', '@', '=']).unwrap_or(after_prefix.len());
            (material_len >= 8).then_some(idx)
        })
        .min()
}

/// JWTs are three `.`-separated base64url segments, header+payload
/// non-trivial in length; this is a shape check (not signature
/// verification), matching the card's "JWT shape" requirement.
fn looks_like_jwt(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    let b64url = |s: &str| {
        !s.is_empty()
            && s.len() >= 8
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    b64url(parts[0]) && b64url(parts[1]) && !parts[2].is_empty()
}

/// FAL `id:secret` shape, ported from doomscrum's `looks_like_fal_key`.
fn looks_like_fal_key(token: &str) -> bool {
    let Some((id, secret)) = token.split_once(':') else {
        return false;
    };
    let hexish = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    id.len() >= 8 && hexish(id) && secret.len() >= 16 && secret.chars().all(|c| c.is_ascii_hexdigit())
}

/// Runs `gitleaks stdin` (if the binary is on PATH) as a second, independent
/// detection pass over already-shape-redacted text, catching high-entropy
/// generic secrets the fixed prefix/shape rules above don't name explicitly
/// (arbitrary API keys with no recognizable prefix). Best-effort: if
/// gitleaks isn't installed or errors, this is a no-op — the shape-based
/// `redact` above is the primary, always-available layer.
pub fn redact_with_gitleaks(text: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let Ok(mut child) = Command::new("gitleaks")
        .args(["stdin", "--report-format", "json", "--report-path", "-", "--exit-code", "0", "--log-level", "fatal"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return text.to_string();
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
    }
    let Ok(output) = child.wait_with_output() else {
        return text.to_string();
    };
    let Ok(findings) = serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout) else {
        return text.to_string();
    };
    let secrets: Vec<String> = findings
        .iter()
        .filter_map(|finding| finding.get("Secret").and_then(|v| v.as_str()))
        .filter(|secret| secret.len() >= 6)
        .map(str::to_string)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    if secrets.is_empty() {
        return text.to_string();
    }
    redact(text, &secrets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_known_prefix_shapes() {
        for token in [
            "sk-or-v1-abcDEF1234567890",
            "ghp_TOKEN9876543210abcdef",
            "github_pat_11ABCDEFG0abcdefghijklmnop",
            "AKIAIOSFODNN7EXAMPLE",
        ] {
            let out = redact(&format!("value is {token} here"), &[]);
            assert!(!out.contains(token), "failed to redact {token}: {out}");
            assert!(out.contains(REDACTED));
        }
        let slack = ["xoxb", "1234567890", "abcdefghijklmnop"].join("-");
        let out = redact(&format!("value is {slack} here"), &[]);
        assert!(!out.contains(&slack), "failed to redact Slack shape: {out}");
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn redacts_real_bearer_token_but_spares_shell_variable_reference() {
        let real = redact("curl -H \"Authorization: Bearer ghp_REALTOKEN1234567890\"", &[]);
        assert!(!real.contains("ghp_REALTOKEN1234567890"), "{real}");
        assert!(real.contains(REDACTED));

        // Dominant false-positive class found in the harness-kit-915
        // transcript scan: a curl command quoting `Bearer $VARNAME` (a
        // shell-variable reference, not a resolved value) must survive
        // untouched.
        let reference = "curl -H \"Authorization: Bearer $POWDER_API_KEY\"";
        assert_eq!(redact(reference, &[]), reference);
        let braced = "curl -H \"Authorization: Bearer ${POWDER_API_KEY}\"";
        assert_eq!(redact(braced, &[]), braced);
    }

    #[test]
    fn redacts_jwt_shape() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let out = redact(&format!("token: {jwt}"), &[]);
        assert!(!out.contains(jwt), "{out}");
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn redacts_pem_private_key_block() {
        let block = "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BA\n-----END PRIVATE KEY-----";
        let text = format!("here is a key:\n{block}\ndone");
        let out = redact(&text, &[]);
        assert!(!out.contains("MIIEvQIBADANBgkqhkiG9w0BA"), "{out}");
        assert!(out.contains(REDACTED));
        assert!(out.starts_with("here is a key:\n"));
        assert!(out.ends_with("\ndone"));
    }

    #[test]
    fn redacts_pem_block_with_no_footer() {
        let text = "leaked: -----BEGIN RSA PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BA";
        let out = redact(text, &[]);
        assert!(!out.contains("MIIEvQIBADANBgkqhkiG9w0BA"), "{out}");
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn redacts_exact_known_value_regardless_of_shape() {
        let secret = "totally-unshaped-plaintext-password-value";
        let out = redact(&format!("printed {secret} oops"), &[secret.to_string()]);
        assert!(!out.contains(secret), "{out}");
    }

    #[test]
    fn leaves_prose_and_urls_untouched() {
        let line = "at 12:34:56 fetched https://queue.fal.run/path ok, sk- is just a prefix word";
        assert_eq!(redact(line, &[]), line);
    }

    #[test]
    fn redacts_secret_glued_to_prefix_text_via_a_hyphen() {
        // Live-verified bug pin: a naive per-segment starts_with(prefix)
        // check missed this exact shape during harness-kit-915's own manual
        // testing — `err-side-` glued directly onto `sk-or-v1-...` via a
        // hyphen made the whole run one token that never *starts with*
        // `sk-`, so the secret sailed through unredacted in a real stderr
        // line (`echo err-side-$KEY >&2`).
        let leaked = "err-side-sk-or-v1-ZZZZ1234567890zz";
        let out = redact(leaked, &[]);
        assert!(!out.contains("ZZZZ1234567890zz"), "{out}");
        assert!(out.contains(REDACTED), "{out}");

        // A bare prefix word with no real secret material after it must
        // still survive untouched (the length floor on the tail segment).
        let benign = "pre-existing sk-users often ask about pricing";
        assert_eq!(redact(benign, &[]), benign);
    }

    #[test]
    fn gitleaks_pass_is_a_noop_when_binary_missing_or_no_findings() {
        // Best-effort contract: clean input round-trips unchanged whether or
        // not gitleaks is actually installed in the test environment.
        let clean = "just some ordinary command output, nothing secret here";
        assert_eq!(redact_with_gitleaks(clean), clean);
    }
}
