use std::fs;

use serde_json::Value;

/// Classifies a Skill-tool invocation as "direct" (the nearest preceding
/// notable transcript event is a genuine human/system-authored user turn —
/// e.g. the user typed `/some-skill` or a natural-language request that
/// triggered this skill first), "routed" (the nearest preceding notable event
/// is another Skill invocation with no fresh user turn in between — e.g.
/// `/design`'s own prose telling the agent to invoke `leon-brutalist-skill`
/// mid-turn), or "unknown" when no transcript is available to classify from.
///
/// This closes the routed-invocation blind spot: without it, a vendored
/// skill only ever reachable via another skill's routing table looks
/// identical in telemetry to one invoked directly, and a zero-count skill
/// could mean either "never used" (strong signal to cull) or "used, but only
/// ever as an invisible sub-call" (weak signal — do not cull).
pub fn classify(data: &Value) -> &'static str {
    let Some(transcript_path) = data.get("transcript_path").and_then(Value::as_str) else {
        return "unknown";
    };
    let Ok(transcript) = fs::read_to_string(transcript_path) else {
        return "unknown";
    };
    classify_from_transcript(&transcript)
}

fn classify_from_transcript(transcript: &str) -> &'static str {
    let entries: Vec<Value> = transcript
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .collect();

    // The current invocation is the last Skill tool_use in the transcript —
    // PostToolUse fires immediately after this call completes, so it is the
    // most recently appended one. Everything before it is prior history.
    let Some(current_index) = entries
        .iter()
        .rposition(|entry| assistant_invokes_skill(transcript_content(entry)))
    else {
        return "unknown";
    };

    for entry in entries[..current_index].iter().rev() {
        let entry_type = entry.get("type").and_then(Value::as_str).unwrap_or("");
        let content = transcript_content(entry);
        match entry_type {
            "assistant" if assistant_invokes_skill(content) => return "routed",
            "user" if is_genuine_user_turn(content) => return "direct",
            _ => {}
        }
    }
    // No prior notable event: this is the first skill call the transcript
    // has any record of, which only happens in direct response to a user turn.
    "direct"
}

fn transcript_content(entry: &Value) -> Option<&Value> {
    entry
        .get("message")
        .and_then(|message| message.get("content"))
}

/// A "genuine" user turn is fresh human/system input — a string prompt, a
/// slash-command marker, or a text block — as opposed to a `tool_result`
/// loopback (the agent's own tool output being fed back to it mid-turn),
/// which is not a new turn at all.
fn is_genuine_user_turn(content: Option<&Value>) -> bool {
    match content {
        Some(Value::String(_)) => true,
        Some(Value::Array(blocks)) => blocks
            .iter()
            .any(|block| block.get("type").and_then(Value::as_str) != Some("tool_result")),
        _ => false,
    }
}

fn assistant_invokes_skill(content: Option<&Value>) -> bool {
    match content {
        Some(Value::Array(blocks)) => blocks.iter().any(|block| {
            block.get("type").and_then(Value::as_str) == Some("tool_use")
                && block.get("name").and_then(Value::as_str) == Some("Skill")
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn transcript(rows: Vec<Value>) -> String {
        rows.into_iter()
            .map(|row| serde_json::to_string(&row).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn direct_invocation_after_genuine_user_message_is_classified_direct() {
        let transcript = transcript(vec![
            json!({"type": "user", "message": {"content": "please run /design brutalist"}}),
            json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Skill", "input": {"skill": "design", "args": "brutalist"}}
            ]}}),
        ]);
        assert_eq!(classify_from_transcript(&transcript), "direct");
    }

    #[test]
    fn nested_invocation_with_no_intervening_user_turn_is_classified_routed() {
        let transcript = transcript(vec![
            json!({"type": "user", "message": {"content": "please run /design brutalist"}}),
            json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Skill", "input": {"skill": "design", "args": "brutalist"}}
            ]}}),
            json!({"type": "user", "message": {"content": [
                {"type": "tool_result", "tool_use_id": "t1", "content": "Launching skill: design"}
            ]}}),
            json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Read", "input": {"file_path": "SKILL.md"}}
            ]}}),
            json!({"type": "user", "message": {"content": [
                {"type": "tool_result", "tool_use_id": "t2", "content": "file contents"}
            ]}}),
            json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Skill", "input": {"skill": "leon-brutalist-skill", "args": ""}}
            ]}}),
        ]);
        assert_eq!(classify_from_transcript(&transcript), "routed");
    }

    #[test]
    fn fresh_user_turn_between_skill_calls_resets_to_direct() {
        let transcript = transcript(vec![
            json!({"type": "user", "message": {"content": "run /harness-engineering"}}),
            json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Skill", "input": {"skill": "harness-engineering", "args": ""}}
            ]}}),
            json!({"type": "user", "message": {"content": "actually, run /shape instead"}}),
            json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "name": "Skill", "input": {"skill": "shape", "args": ""}}
            ]}}),
        ]);
        assert_eq!(classify_from_transcript(&transcript), "direct");
    }

    #[test]
    fn first_ever_skill_call_with_no_prior_history_is_direct() {
        let transcript = serde_json::to_string(&json!({
            "type": "assistant",
            "message": {"content": [
                {"type": "tool_use", "name": "Skill", "input": {"skill": "orient", "args": ""}}
            ]}
        }))
        .unwrap();
        assert_eq!(classify_from_transcript(&transcript), "direct");
    }

    #[test]
    fn empty_or_garbage_transcript_is_unknown() {
        assert_eq!(classify_from_transcript(""), "unknown");
        assert_eq!(classify_from_transcript("not json\nat all"), "unknown");
    }

    #[test]
    fn missing_transcript_path_is_unknown() {
        assert_eq!(classify(&json!({})), "unknown");
    }

    #[test]
    fn unreadable_transcript_path_is_unknown() {
        assert_eq!(
            classify(&json!({"transcript_path": "/nonexistent/path.jsonl"})),
            "unknown"
        );
    }
}
