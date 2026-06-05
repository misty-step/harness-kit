# Case: Session Start Orientation

Prompt:

> /orient

Fixture/context:

- Current repo is Harness Kit.
- `git status --short --branch --untracked-files=all` shows a clean feature
  branch.
- `backlog.d/` has no active ticket files.
- Recent commits include a skill or harness primitive change.
- `.harness-kit/agents.yaml` exists.

Expected report:

- Uses the `**Orientation**` heading.
- Names repo/branch, workspace state, current focus, backlog signal, recent
  closure signal, roster state, constraints, blockers/gaps, likely next skill,
  and residual uncertainty.
- Cites live paths or commands such as `AGENTS.md`, `project.md`, `git status`,
  `git log`, `backlog.d/`, and `.harness-kit/agents.yaml`.
- Recommends `/groom` when backlog is empty or `/deliver` when a shaped active
  ticket exists.
- Does not score readiness, mine transcripts, create session memory, or perform
  broad debrief/reflection.

