#!/usr/bin/env python3
"""Collect deterministic evidence for /tailor's post-install audit.

The collector writes facts for a critic. It does not decide whether a
tailored harness is good.
"""

from __future__ import annotations

import argparse
import filecmp
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ALWAYS_INSTALL = [
    "research",
    "groom",
    "shape",
    "implement",
    "qa",
    "demo",
    "code-review",
    "refactor",
    "ci",
    "diagnose",
    "monitor",
    "deliver",
    "settle",
    "ship",
    "trace",
    "yeet",
    "flywheel",
]

WORKFLOW_SKILLS = set(ALWAYS_INSTALL + ["deploy"])
LIFECYCLE_SKILLS = ["groom", "ship", "settle", "trace", "flywheel", "implement", "deliver"]
GATE_SURFACES = ["ci", "implement", "deliver", "settle", "ship", "qa", "monitor"]
HARNESS_SKILL_DIRS = [".claude/skills", ".codex/skills", ".pi/skills"]
AGENTS_REQUIRED_SECTIONS = [
    "Stack & boundaries",
    "Gate contract",
    "Lifecycle",
    "Known debt",
    "Harness index",
    "Invariants",
]
TEXT_SUFFIXES = {
    ".md",
    ".txt",
    ".yaml",
    ".yml",
    ".json",
    ".toml",
    ".sh",
    ".py",
}


@dataclass(frozen=True)
class Roots:
    repo: Path
    spellbook: Path
    shared_skills: Path | None
    agents: Path | None
    repo_brief: Path | None
    output: Path
    run_id: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--repo", dest="repo_root_option", help="target repo root")
    parser.add_argument("repo_root", nargs="?", default=".", help="target repo root")
    parser.add_argument("--spellbook-root", help="canonical spellbook checkout")
    parser.add_argument("--shared-skill-root", help="repo-local shared skill root")
    parser.add_argument("--agent-root", help="installed agent root")
    parser.add_argument("--repo-brief", help="repo brief path")
    parser.add_argument("--run-id", help="audit run id; default is UTC timestamp")
    parser.add_argument("--output-dir", help="directory for evidence.json and artifacts")
    return parser.parse_args()


def run(cmd: list[str], cwd: Path) -> str:
    return subprocess.check_output(cmd, cwd=cwd, text=True, stderr=subprocess.DEVNULL).strip()


def git_root(path: Path) -> Path:
    try:
        return Path(run(["git", "rev-parse", "--show-toplevel"], path)).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return path.resolve()


def discover_spellbook(script_path: Path, explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    current = script_path.resolve()
    for parent in [current.parent, *current.parents]:
        if (parent / "skills" / "tailor" / "SKILL.md").is_file() and (parent / "agents").is_dir():
            return parent
    return git_root(Path.cwd())


def first_existing(paths: list[Path]) -> Path | None:
    for path in paths:
        if path.exists():
            return path
    return None


def build_roots(args: argparse.Namespace) -> Roots:
    repo = git_root(Path(args.repo_root_option or args.repo_root))
    spellbook = discover_spellbook(Path(__file__), args.spellbook_root)
    shared = resolve_repo_path(repo, args.shared_skill_root) if args.shared_skill_root else first_existing(
        [repo / ".agent" / "skills", repo / ".agents" / "skills"]
    )
    agents = resolve_repo_path(repo, args.agent_root) if args.agent_root else first_existing(
        [repo / ".agents" / "agents", repo / ".claude" / "agents", repo / "agents"]
    )
    brief = resolve_repo_path(repo, args.repo_brief) if args.repo_brief else first_existing(
        [repo / ".spellbook" / "repo-brief.md", repo / ".claude" / ".tailor" / "repo-brief.md"]
    )
    run_id = args.run_id or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    output = resolve_repo_path(repo, args.output_dir) if args.output_dir else repo / ".spellbook" / "tailor" / "audit" / run_id
    return Roots(repo=repo, spellbook=spellbook, shared_skills=shared, agents=agents, repo_brief=brief, output=output, run_id=run_id)


def resolve_repo_path(repo: Path, value: str) -> Path:
    path = Path(value)
    return path.resolve() if path.is_absolute() else (repo / path).resolve()


def rel(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path)


def read_text(path: Path) -> str:
    return path.read_text(errors="replace")


def marker(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    result: dict[str, str] = {}
    for line in read_text(path).splitlines():
        if ":" not in line or line.lstrip().startswith("#"):
            continue
        key, value = line.split(":", 1)
        result[key.strip()] = value.strip()
    return result


def text_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if path.is_file() and path.suffix in TEXT_SUFFIXES:
            files.append(path)
        elif path.is_dir():
            files.extend(child for child in path.rglob("*") if child.is_file() and child.suffix in TEXT_SUFFIXES)
    return sorted(set(files))


def grep(pattern: re.Pattern[str], paths: list[Path], root: Path) -> list[dict[str, Any]]:
    hits: list[dict[str, Any]] = []
    for path in text_files(paths):
        try:
            lines = read_text(path).splitlines()
        except OSError:
            continue
        for lineno, line in enumerate(lines, 1):
            if pattern.search(line):
                hits.append({"path": rel(path, root), "line": lineno, "text": line.strip()[:240]})
    return hits


def installed_skills(roots: Roots) -> list[dict[str, Any]]:
    if not roots.shared_skills or not roots.shared_skills.is_dir():
        return []
    skills: list[dict[str, Any]] = []
    for entry in sorted(roots.shared_skills.iterdir(), key=lambda item: item.name):
        if not entry.is_dir() and not entry.is_symlink():
            continue
        marker_path = entry / ".spellbook" if entry.is_dir() else entry.with_suffix(entry.suffix + ".spellbook")
        if entry.is_symlink():
            marker_path = entry.parent / f"{entry.name}.spellbook"
        data = marker(marker_path)
        source = data.get("source", entry.name)
        installed_skill = entry / "SKILL.md"
        source_skill = roots.spellbook / "skills" / source / "SKILL.md"
        byte_identical = (
            installed_skill.is_file()
            and source_skill.is_file()
            and filecmp.cmp(installed_skill, source_skill, shallow=False)
        )
        skills.append(
            {
                "name": entry.name,
                "path": rel(entry, roots.repo),
                "category": data.get("category"),
                "source": source,
                "installed_by": data.get("installed-by"),
                "is_symlink": entry.is_symlink(),
                "symlink_target": os.readlink(entry) if entry.is_symlink() else None,
                "marker_path": rel(marker_path, roots.repo) if marker_path.exists() else None,
                "has_skill_md": installed_skill.is_file(),
                "byte_identical_to_source": byte_identical,
            }
        )
    return skills


def bridge_topology(roots: Roots, skill_names: list[str]) -> list[dict[str, Any]]:
    bridges: list[dict[str, Any]] = []
    for bridge_dir in HARNESS_SKILL_DIRS:
        base = roots.repo / bridge_dir
        if not base.exists():
            bridges.append({"dir": bridge_dir, "exists": False, "entries": []})
            continue
        entries: list[dict[str, Any]] = []
        for name in skill_names:
            path = base / name
            entries.append(
                {
                    "name": name,
                    "path": rel(path, roots.repo),
                    "exists": path.exists() or path.is_symlink(),
                    "is_symlink": path.is_symlink(),
                    "target": os.readlink(path) if path.is_symlink() else None,
                    "broken": path.is_symlink() and not path.exists(),
                }
            )
        bridges.append({"dir": bridge_dir, "exists": True, "entries": entries})
    return bridges


def agent_refs(roots: Roots, scan_roots: list[Path]) -> dict[str, Any]:
    pattern = re.compile(r"subagent_type\s*[:=]\s*[`'\"]?([A-Za-z0-9_-]+)")
    refs = sorted({hit["text"].split("subagent_type", 1)[-1].strip(" :=`'\"") for hit in grep(pattern, scan_roots, roots.repo)})
    resolved = []
    for name in refs:
        agent_path = roots.agents / f"{name}.md" if roots.agents else None
        resolved.append({"name": name, "resolves": bool(agent_path and agent_path.is_file()), "path": rel(agent_path, roots.repo) if agent_path else None})
    return {"root": rel(roots.agents, roots.repo) if roots.agents else None, "refs": resolved}


def repo_brief_facts(roots: Roots) -> dict[str, Any]:
    if not roots.repo_brief or not roots.repo_brief.is_file():
        return {"path": None, "exists": False}
    text = read_text(roots.repo_brief)
    debt_ids = sorted(set(re.findall(r"\b\d{3}-[a-z0-9-]+\.md\b", text)))
    gate = "dagger call check --source=." if "dagger call check --source=." in text else None
    return {
        "path": rel(roots.repo_brief, roots.repo),
        "exists": True,
        "generated": next((line.split(":", 1)[1].strip() for line in text.splitlines() if line.startswith("generated:")), None),
        "load_bearing_gate": gate,
        "says_no_deploy_target": "no deploy target" in text.lower(),
        "debt_ids": debt_ids,
    }


def agents_md_facts(roots: Roots) -> dict[str, Any]:
    path = roots.repo / "AGENTS.md"
    if not path.is_file():
        return {"path": rel(path, roots.repo), "exists": False}
    text = read_text(path)
    headings = [
        line.removeprefix("##").strip()
        for line in text.splitlines()
        if line.startswith("## ") and not line.startswith("###")
    ]
    words = re.findall(r"\b[\w'-]+\b", text)
    missing = [section for section in AGENTS_REQUIRED_SECTIONS if section not in headings]
    return {
        "path": rel(path, roots.repo),
        "exists": True,
        "top_level_headings": headings,
        "top_level_heading_count": len(headings),
        "word_count": len(words),
        "required_sections": AGENTS_REQUIRED_SECTIONS,
        "missing_required_sections": missing,
        "too_many_headings": len(headings) > 6,
        "too_many_words": len(words) > 650,
        "has_unfiled_debt": bool(re.search(r"\(unfiled\)", text, re.I)),
    }


def lifecycle_facts(roots: Roots) -> dict[str, list[dict[str, Any]]]:
    if not roots.shared_skills:
        return {}
    terms = re.compile(r"\b(backlog\.d|_done|Closes-backlog|Ships-backlog|trace|work\.jsonl|verdict|archive|detector|git trailer|GitHub Issues)\b", re.I)
    facts: dict[str, list[dict[str, Any]]] = {}
    for name in LIFECYCLE_SKILLS:
        path = roots.shared_skills / name / "SKILL.md"
        facts[name] = grep(terms, [path], roots.repo) if path.is_file() else []
    return facts


def shared_script_status(roots: Roots) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for name in ["backlog.sh", "verdicts.sh"]:
        repo_script = roots.repo / "scripts" / "lib" / name
        source_script = roots.spellbook / "scripts" / "lib" / name
        rows.append(
            {
                "path": rel(repo_script, roots.repo),
                "exists": repo_script.is_file(),
                "source_exists": source_script.is_file(),
                "matches_source": repo_script.is_file()
                and source_script.is_file()
                and filecmp.cmp(repo_script, source_script, shallow=False),
            }
        )
    return rows


def write_artifacts(
    roots: Roots,
    portable_hits: list[dict[str, Any]],
    lifecycle_hits: list[dict[str, Any]],
    bridges: list[dict[str, Any]],
    byte_identical: list[str],
) -> dict[str, str]:
    grep_dir = roots.output / "grep"
    grep_dir.mkdir(parents=True, exist_ok=True)
    portable_path = grep_dir / "portable-paths.txt"
    lifecycle_path = grep_dir / "lifecycle.txt"
    readlinks_path = roots.output / "readlinks.txt"
    byte_identity_path = roots.output / "byte-identity.txt"
    portable_path.write_text("\n".join(f"{hit['path']}:{hit['line']}: {hit['text']}" for hit in portable_hits) + ("\n" if portable_hits else ""))
    lifecycle_path.write_text("\n".join(f"{hit['path']}:{hit['line']}: {hit['text']}" for hit in lifecycle_hits) + ("\n" if lifecycle_hits else ""))
    readlink_lines = [
        f"{entry['path']} -> {entry['target'] or '<not-symlink>'}{' BROKEN' if entry['broken'] else ''}"
        for bridge in bridges
        for entry in bridge["entries"]
        if entry["exists"]
    ]
    readlinks_path.write_text("\n".join(readlink_lines) + ("\n" if readlink_lines else ""))
    byte_identity_path.write_text("\n".join(byte_identical) + ("\n" if byte_identical else ""))
    return {
        "portable_paths": rel(portable_path, roots.repo),
        "lifecycle": rel(lifecycle_path, roots.repo),
        "readlinks": rel(readlinks_path, roots.repo),
        "byte_identity": rel(byte_identity_path, roots.repo),
    }


def collect(roots: Roots) -> dict[str, Any]:
    skills = installed_skills(roots)
    skill_names = [item["name"] for item in skills]
    scan_roots = [roots.repo / "AGENTS.md", roots.repo / "CLAUDE.md"]
    if roots.repo_brief:
        scan_roots.append(roots.repo_brief)
    if roots.shared_skills:
        scan_roots.append(roots.shared_skills)

    portable_hits = grep(re.compile(r"(/Users/[A-Za-z0-9_-]+/|C:\\Users\\[A-Za-z0-9_-]+\\)"), scan_roots, roots.repo)
    brief_facts = repo_brief_facts(roots)
    deploy_pattern = re.compile(r"\b(/deploy|deploy receipt|post-deploy|watch the deploy)\b", re.I)
    lifecycle_scan_roots = [roots.repo / "AGENTS.md", roots.repo / "CLAUDE.md"]
    if roots.shared_skills:
        lifecycle_scan_roots.append(roots.shared_skills)
    lifecycle_hits = grep(deploy_pattern, lifecycle_scan_roots, roots.repo) if brief_facts.get("says_no_deploy_target") else []
    missing_always = [name for name in ALWAYS_INSTALL if name not in skill_names]
    bridges = bridge_topology(roots, skill_names or ALWAYS_INSTALL)
    byte_identical = [
        item["name"] for item in skills if item["name"] in WORKFLOW_SKILLS and item["byte_identical_to_source"]
    ]
    artifacts = write_artifacts(roots, portable_hits, lifecycle_hits, bridges, byte_identical)

    gate_pattern = re.compile(r"(dagger call check --source=\.|pnpm|npm|pytest|bun test|cargo test|go test|mix test)")
    gate_inventory = {surface: grep(gate_pattern, [roots.shared_skills / surface / "SKILL.md"], roots.repo) if roots.shared_skills else [] for surface in GATE_SURFACES}
    gate_inventory["AGENTS.md"] = grep(gate_pattern, [roots.repo / "AGENTS.md"], roots.repo)

    return {
        "schema_version": 1,
        "run_id": roots.run_id,
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "repo_root": str(roots.repo),
        "spellbook_root": str(roots.spellbook),
        "shared_skill_root": rel(roots.shared_skills, roots.repo) if roots.shared_skills else None,
        "agent_root": rel(roots.agents, roots.repo) if roots.agents else None,
        "artifacts": artifacts,
        "installed_skills": skills,
        "always_install": {"expected": ALWAYS_INSTALL, "missing": missing_always, "trace_present": "trace" in skill_names},
        "bridges": bridges,
        "agent_refs": agent_refs(roots, scan_roots),
        "portable_path_hits": portable_hits,
        "stale_lifecycle_hits": lifecycle_hits,
        "gate_inventory": gate_inventory,
        "lifecycle_facts": lifecycle_facts(roots),
        "repo_brief": brief_facts,
        "agents_debt": {
            "unfiled_hits": grep(re.compile(r"\(unfiled\)", re.I), [roots.repo / "AGENTS.md"], roots.repo),
            "debt_ids": sorted(set(re.findall(r"\b\d{3}-[a-z0-9-]+\.md\b", read_text(roots.repo / "AGENTS.md") if (roots.repo / "AGENTS.md").is_file() else ""))),
        },
        "agents_md": agents_md_facts(roots),
        "shared_scripts": shared_script_status(roots),
        "workflow_byte_identical": byte_identical,
    }


def main() -> int:
    args = parse_args()
    roots = build_roots(args)
    if not roots.repo.is_dir():
        print(f"repo root not found: {roots.repo}", file=sys.stderr)
        return 2
    if not roots.spellbook.is_dir():
        print(f"spellbook root not found: {roots.spellbook}", file=sys.stderr)
        return 2
    roots.output.mkdir(parents=True, exist_ok=True)
    evidence = collect(roots)
    out = roots.output / "evidence.json"
    out.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    print(rel(out, roots.repo))
    return 0


if __name__ == "__main__":
    sys.exit(main())
