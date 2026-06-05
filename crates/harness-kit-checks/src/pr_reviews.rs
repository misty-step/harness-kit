use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrReviewBundle {
    pub pr: Value,
    pub reviews: Vec<Value>,
    pub inline_comments: Vec<Value>,
    pub issue_comments: Vec<Value>,
}

pub fn fetch(selector: Option<&str>) -> Result<PrReviewBundle> {
    let mut args = vec![
        "pr",
        "view",
        "--json",
        "number,title,url,baseRefName,headRefName",
    ];
    if let Some(selector) = selector {
        args.insert(2, selector);
    }
    let pr = gh_json(&args)?;
    let pr_number = value_to_string(&pr["number"]);
    let repo = repo_from_pr_url(pr["url"].as_str().unwrap_or_default())?;
    let reviews = gh_paginated_array(&format!(
        "/repos/{repo}/pulls/{pr_number}/reviews?per_page=100"
    ))?;
    let inline_comments = gh_paginated_array(&format!(
        "/repos/{repo}/pulls/{pr_number}/comments?per_page=100"
    ))?;
    let issue_comments = gh_paginated_array(&format!(
        "/repos/{repo}/issues/{pr_number}/comments?per_page=100"
    ))?;
    Ok(PrReviewBundle {
        pr,
        reviews,
        inline_comments,
        issue_comments,
    })
}

pub fn render(bundle: &PrReviewBundle) -> String {
    let mut output = String::new();
    let pr_number = value_to_string(&bundle.pr["number"]);
    let pr_title = string_at(&bundle.pr, &["title"]).unwrap_or_default();
    output.push_str(&format!("# PR {pr_number}: {pr_title}\n"));
    output.push_str(&format!(
        "URL: {}\n",
        string_at(&bundle.pr, &["url"]).unwrap_or_default()
    ));
    output.push_str(&format!(
        "Base: {}\n",
        string_at(&bundle.pr, &["baseRefName"]).unwrap_or_default()
    ));
    output.push_str(&format!(
        "Head: {}\n\n",
        string_at(&bundle.pr, &["headRefName"]).unwrap_or_default()
    ));

    output.push_str("## Review Summaries\n\n");
    if bundle.reviews.is_empty() {
        output.push_str("_none_\n");
    } else {
        let mut reviews = bundle.reviews.clone();
        reviews.sort_by_key(|review| {
            string_at(review, &["submitted_at"])
                .or_else(|| string_at(review, &["created_at"]))
                .unwrap_or_default()
                .to_string()
        });
        for review in reviews {
            let id = value_to_string(&review["id"]);
            let user = string_at(&review, &["user", "login"]).unwrap_or_default();
            let state = string_at(&review, &["state"]).unwrap_or_default();
            let when = string_at(&review, &["submitted_at"])
                .or_else(|| string_at(&review, &["submittedAt"]))
                .or_else(|| string_at(&review, &["created_at"]))
                .unwrap_or_default();
            output.push_str(&format!("### Review #{id} - {user} - {state} - {when}\n"));
            output.push_str(&format!(
                "Commit: {}\n",
                string_at(&review, &["commit_id"]).unwrap_or("n/a")
            ));
            output.push_str(&format!(
                "URL: {}\n\n",
                string_at(&review, &["html_url"]).unwrap_or("n/a")
            ));
            output.push_str(nonempty_or(&review, "body", "_no body_"));
            output.push_str("\n\n");
        }
    }

    output.push_str("\n## Inline Review Comments\n\n");
    if bundle.inline_comments.is_empty() {
        output.push_str("_none_\n");
    } else {
        let mut comments = bundle.inline_comments.clone();
        comments.sort_by_key(|comment| {
            string_at(comment, &["created_at"])
                .unwrap_or_default()
                .to_string()
        });
        for comment in comments {
            let id = value_to_string(&comment["id"]);
            let user = string_at(&comment, &["user", "login"]).unwrap_or_default();
            let path = string_at(&comment, &["path"]).unwrap_or_default();
            let line = value_to_string(
                comment
                    .get("line")
                    .filter(|value| !value.is_null())
                    .or_else(|| {
                        comment
                            .get("original_line")
                            .filter(|value| !value.is_null())
                    })
                    .unwrap_or(&Value::String("n/a".to_string())),
            );
            output.push_str(&format!("### Comment #{id} - {user} - {path}:{line}\n"));
            output.push_str(&format!(
                "Created: {}\n",
                string_at(&comment, &["created_at"]).unwrap_or_default()
            ));
            output.push_str(&format!(
                "Review ID: {}\n",
                value_or_na(&comment, "pull_request_review_id")
            ));
            output.push_str(&format!(
                "In reply to: {}\n",
                value_or_na(&comment, "in_reply_to_id")
            ));
            output.push_str(&format!(
                "URL: {}\n\n",
                string_at(&comment, &["html_url"]).unwrap_or("n/a")
            ));
            if let Some(diff_hunk) = string_at(&comment, &["diff_hunk"])
                && !diff_hunk.is_empty()
            {
                output.push_str(&format!("```diff\n{diff_hunk}\n```\n\n"));
            }
            output.push_str(nonempty_or(&comment, "body", "_no body_"));
            output.push_str("\n\n");
        }
    }

    output.push_str("\n## PR Conversation Comments\n\n");
    if bundle.issue_comments.is_empty() {
        output.push_str("_none_\n");
    } else {
        let mut comments = bundle.issue_comments.clone();
        comments.sort_by_key(|comment| {
            string_at(comment, &["created_at"])
                .unwrap_or_default()
                .to_string()
        });
        for comment in comments {
            let id = value_to_string(&comment["id"]);
            let user = string_at(&comment, &["user", "login"]).unwrap_or_default();
            let created = string_at(&comment, &["created_at"]).unwrap_or_default();
            output.push_str(&format!("### Comment #{id} - {user} - {created}\n"));
            output.push_str(&format!(
                "URL: {}\n\n",
                string_at(&comment, &["html_url"]).unwrap_or("n/a")
            ));
            output.push_str(nonempty_or(&comment, "body", "_no body_"));
            output.push_str("\n\n");
        }
    }
    output
}

pub fn repo_from_pr_url(url: &str) -> Result<String> {
    let pattern = Regex::new(r"https?://[^/]+/(?P<repo>[^/]+/[^/]+)/pull/").unwrap();
    pattern
        .captures(url)
        .and_then(|captures| captures.name("repo"))
        .map(|repo| repo.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("failed to infer repo from PR URL: {url}"))
}

fn gh_json(args: &[&str]) -> Result<Value> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .with_context(|| "gh CLI is required")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn gh_paginated_array(endpoint: &str) -> Result<Vec<Value>> {
    let output = Command::new("gh")
        .args(["api", "--paginate", endpoint])
        .output()
        .with_context(|| "gh CLI is required")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if let Ok(Value::Array(array)) = serde_json::from_str::<Value>(trimmed) {
        return Ok(array);
    }
    let mut items = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<Value>(line)? {
            Value::Array(array) => items.extend(array),
            value => items.push(value),
        }
    }
    Ok(items)
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str()
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "n/a".to_string(),
        Value::String(value) => value.clone(),
        value => value.to_string(),
    }
}

fn value_or_na(value: &Value, key: &str) -> String {
    value
        .get(key)
        .filter(|value| !value.is_null())
        .map(value_to_string)
        .unwrap_or_else(|| "n/a".to_string())
}

fn nonempty_or<'a>(value: &'a Value, key: &str, fallback: &'a str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|body| !body.is_empty())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn renders_full_pr_review_markdown() {
        let bundle = PrReviewBundle {
            pr: json!({
                "number": 42,
                "title": "Tighten gates",
                "url": "https://github.com/acme/harness-kit/pull/42",
                "baseRefName": "main",
                "headRefName": "feature"
            }),
            reviews: vec![
                json!({"id": 2, "user": {"login": "later"}, "state": "COMMENTED", "created_at": "2026-01-02T00:00:00Z", "body": ""}),
                json!({"id": 1, "user": {"login": "first"}, "state": "APPROVED", "submitted_at": "2026-01-01T00:00:00Z", "commit_id": "abc", "html_url": "https://review", "body": "Looks good"}),
            ],
            inline_comments: vec![json!({
                "id": 10,
                "user": {"login": "reviewer"},
                "path": "src/lib.rs",
                "original_line": 7,
                "created_at": "2026-01-03T00:00:00Z",
                "pull_request_review_id": 1,
                "html_url": "https://inline",
                "diff_hunk": "@@ -1 +1 @@",
                "body": "Please adjust"
            })],
            issue_comments: vec![json!({
                "id": 20,
                "user": {"login": "commenter"},
                "created_at": "2026-01-04T00:00:00Z",
                "html_url": "https://issue",
                "body": ""
            })],
        };

        let output = render(&bundle);

        assert!(output.contains("# PR 42: Tighten gates"));
        assert!(output.contains("### Review #1 - first - APPROVED - 2026-01-01T00:00:00Z"));
        assert!(output.contains("### Review #2 - later - COMMENTED - 2026-01-02T00:00:00Z"));
        assert!(output.contains("_no body_"));
        assert!(output.contains("```diff\n@@ -1 +1 @@\n```"));
        assert!(output.contains("Review ID: 1"));
        assert!(output.contains("In reply to: n/a"));
        assert!(output.contains("### Comment #20 - commenter - 2026-01-04T00:00:00Z"));
    }

    #[test]
    fn empty_sections_render_none_marker() {
        let bundle = PrReviewBundle {
            pr: json!({"number": 1, "title": "Empty", "url": "", "baseRefName": "main", "headRefName": "head"}),
            reviews: Vec::new(),
            inline_comments: Vec::new(),
            issue_comments: Vec::new(),
        };

        let output = render(&bundle);

        assert_eq!(output.matches("_none_").count(), 3);
    }

    #[test]
    fn extracts_repo_from_pr_url() {
        assert_eq!(
            repo_from_pr_url("https://github.com/acme/harness-kit/pull/42").unwrap(),
            "acme/harness-kit"
        );
    }
}
