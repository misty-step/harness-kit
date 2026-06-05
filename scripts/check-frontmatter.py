#!/usr/bin/env python3
"""Validate SKILL.md and agent .md frontmatter: required fields, line limits."""

import os
import re
import sys
from collections import defaultdict

import yaml


TRIGGER_LINE_RE = re.compile(r"\bTriggers?:\s*(.*)", re.IGNORECASE)
USE_WHEN_RE = re.compile(r"\bUse (?:when|for):", re.IGNORECASE)
QUOTED_PHRASE_RE = re.compile(r'"([^"]+)"')
FRONTMATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", re.DOTALL)


def parse_frontmatter(content):
    match = FRONTMATTER_RE.match(content)
    if not match:
        return None, "malformed frontmatter"
    try:
        parsed = yaml.safe_load(match.group(1))
    except yaml.YAMLError as e:
        return None, f"invalid YAML frontmatter: {e}"
    if not parsed or not isinstance(parsed, dict):
        return None, "empty frontmatter"
    return parsed, None


def check_frontmatter(path, required_fields=("name", "description"), max_lines=None):
    """Check a single markdown file's YAML frontmatter. Returns list of errors."""
    errors = []
    with open(path) as f:
        content = f.read()
    lines = content.splitlines()

    if max_lines and len(lines) > max_lines:
        errors.append(f"{path}: {len(lines)} lines (max {max_lines})")

    if not content.startswith("---"):
        return [f"{path}: missing frontmatter"]
    fm, error = parse_frontmatter(content)
    if error:
        return [f"{path}: {error}"]

    for field in required_fields:
        if field not in fm:
            errors.append(f"{path}: missing '{field}' in frontmatter")
    return errors


def load_frontmatter(path):
    with open(path) as f:
        content = f.read()
    if not content.startswith("---"):
        return None
    parsed, error = parse_frontmatter(content)
    return None if error else parsed


def normalize_trigger_claim(value):
    value = value.strip().strip(".").strip('"')
    value = re.sub(r"\s*\([^)]*\)\s*$", "", value)
    value = re.sub(r"\s+", " ", value)
    return value.lower()


def collision_key(value):
    value = normalize_trigger_claim(value).lstrip("/")
    value = re.sub(r"[^a-z0-9]+", " ", value)
    return value.strip()


def split_trigger_values(text):
    return [
        normalize_trigger_claim(item)
        for item in text.split(",")
        if normalize_trigger_claim(item)
    ]


def explicit_triggers(description):
    lines = description.splitlines()
    triggers = []
    for index, line in enumerate(lines):
        match = TRIGGER_LINE_RE.search(line)
        if not match:
            continue
        trigger_text = match.group(1)
        for continuation in lines[index + 1 :]:
            if USE_WHEN_RE.search(continuation) or TRIGGER_LINE_RE.search(continuation):
                break
            trigger_text = f"{trigger_text} {continuation.strip()}"
        triggers.extend(split_trigger_values(trigger_text))
    return triggers


def use_when_phrases(description):
    phrases = []
    in_block = False
    for line in description.splitlines():
        if USE_WHEN_RE.search(line):
            in_block = True
        elif TRIGGER_LINE_RE.search(line) or not line.strip():
            in_block = False
        if in_block:
            phrases.extend(
                normalize_trigger_claim(match)
                for match in QUOTED_PHRASE_RE.findall(line)
            )
    return phrases


def trigger_claims(description):
    return explicit_triggers(description) + use_when_phrases(description)


def check_trigger_contracts(skill_frontmatters):
    """Return (errors, warnings) for trigger metadata across first-party skills."""
    warnings = []
    errors = []
    claims = defaultdict(list)
    for path, frontmatter in skill_frontmatters:
        description = frontmatter.get("description") or ""
        triggers = explicit_triggers(description)
        if not triggers:
            errors.append(f"{path}: missing Trigger definition in description")
        for claim in trigger_claims(description):
            key = collision_key(claim)
            if key:
                claims[key].append(path)

    for claim, owners in sorted(claims.items()):
        unique_owners = sorted(set(owners))
        if len(unique_owners) > 1:
            errors.append(
                f"trigger claim collision '{claim}': {', '.join(unique_owners)}"
            )
    return errors, warnings


def main():
    errors = []
    warnings = []
    skill_frontmatters = []

    for name in sorted(os.listdir("skills")):
        path = f"skills/{name}/SKILL.md"
        if os.path.isfile(path):
            errors.extend(check_frontmatter(path, max_lines=500))
            frontmatter = load_frontmatter(path)
            if frontmatter:
                skill_frontmatters.append((path, frontmatter))

    for name in sorted(os.listdir("agents")):
        if name.endswith(".md"):
            errors.extend(check_frontmatter(f"agents/{name}"))

    trigger_errors, trigger_warnings = check_trigger_contracts(skill_frontmatters)
    errors.extend(trigger_errors)
    warnings.extend(trigger_warnings)

    for warning in warnings:
        print(f"WARN: {warning}", file=sys.stderr)

    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        sys.exit(1)
    print("OK: all frontmatter valid")


if __name__ == "__main__":
    main()
