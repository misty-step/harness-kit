#!/usr/bin/env python3
"""Render a redacted local agent transcript as a collapsible Markdown block."""

from __future__ import annotations

import argparse
import html
import re
import sys
from pathlib import Path


MARKER_START = "<!-- harness-kit-agent-transcript:start -->"
MARKER_END = "<!-- harness-kit-agent-transcript:end -->"

REDACTIONS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----", re.S), "[REDACTED_PRIVATE_KEY]"),
    (re.compile(r"\b(?:sk|rk|ghp|glpat|xox[abprs]?)-[A-Za-z0-9_\-]{16,}\b"), "[REDACTED_TOKEN]"),
    (re.compile(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"), "[REDACTED_GITHUB_PAT]"),
    (re.compile(r"\bAKIA[0-9A-Z]{16}\b"), "[REDACTED_AWS_KEY]"),
    (re.compile(r"(?i)\b(cookie|authorization|x-api-key|api[_-]?key|token|password)\s*[:=]\s*[^\s`'\"]{8,}"), r"\1=[REDACTED_SECRET]"),
    (re.compile(r"https?://[^\s)>\"]*(?:code|token|access_token|refresh_token)=[^\s)>\"]+"), "[REDACTED_AUTH_URL]"),
    (re.compile(r"/Users/[A-Za-z0-9_.-]+/"), "~/"),
]

BLOCKED_AFTER_REDACTION: list[re.Pattern[str]] = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\b(?:sk|rk|ghp|glpat|xox[abprs]?)-[A-Za-z0-9_\-]{16,}\b"),
    re.compile(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"(?i)\b(cookie|authorization|x-api-key|api[_-]?key|token|password)\s*[:=]\s*(?!\[REDACTED)[^\s`'\"]{8,}"),
]

DROP_LINE_PATTERNS = [
    re.compile(r"^\s*(system|developer)\b", re.I),
    re.compile(r"(?i)\b(raw tool output|environment dump|export -p|^env$|^set$)\b"),
]


def redact(text: str) -> str:
    out = text.replace("\r\n", "\n")
    kept: list[str] = []
    for line in out.splitlines():
        if any(pattern.search(line) for pattern in DROP_LINE_PATTERNS):
            continue
        kept.append(line)
    out = "\n".join(kept)
    for pattern, replacement in REDACTIONS:
        out = pattern.sub(replacement, out)
    return out.strip()


def assert_safe(text: str) -> None:
    for pattern in BLOCKED_AFTER_REDACTION:
        if pattern.search(text):
            raise SystemExit(f"unsafe transcript: unresolved secret pattern {pattern.pattern!r}")


def render_block(text: str, title: str) -> str:
    safe = redact(text)
    assert_safe(safe)
    body = html.escape(safe, quote=False)
    return (
        f"{MARKER_START}\n"
        "<details>\n"
        f"<summary>{html.escape(title)}</summary>\n\n"
        "```text\n"
        f"{body}\n"
        "```\n\n"
        "</details>\n"
        f"{MARKER_END}\n"
    )


def read_input(path: str | None) -> str:
    if path:
        return Path(path).read_text(encoding="utf-8")
    return sys.stdin.read()


def self_test() -> None:
    sample = """user: fix the bug
assistant: running tests
Authorization: Bearer sk-test_1234567890abcdef
path: /Users/alice/project/file.py
-----BEGIN PRIVATE KEY-----
abc
-----END PRIVATE KEY-----
tool: pytest passed
"""
    rendered = render_block(sample, "Self Test")
    assert "[REDACTED_TOKEN]" in rendered
    assert "[REDACTED_PRIVATE_KEY]" in rendered
    assert "~/project/file.py" in rendered
    assert "sk-test" not in rendered
    print("agent-transcript self-test ok")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", nargs="?", default="render", choices=["render"])
    parser.add_argument("--input", "-i", default=None)
    parser.add_argument("--output", "-o", default=None)
    parser.add_argument("--title", default="Agent Transcript")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0

    rendered = render_block(read_input(args.input), args.title)
    if args.output:
        Path(args.output).write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
