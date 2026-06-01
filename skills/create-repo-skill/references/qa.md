# QA Skill Generator

Generate `.agents/skills/qa/` when the repo needs durable QA paths.

## Lanes

Dispatch in parallel:

| Lane | Objective | Output |
|---|---|---|
| app mapper | identify app shape, commands, ports, packages, deployment targets | fact table |
| surface mapper | routes, endpoints, commands, public APIs, auth requirements | surface table |
| context scout | test infra, QA tools, fixtures, test accounts from safe docs only | fact table |
| critic | attack draft for generic wording and missing live evidence | blockers |

## User Questions

Ask only for truth the repo cannot know:

- Which surface matters most right now?
- Local, preview, staging, or production target?
- Are write-capable flows allowed? If yes, what tenant/account is safe?
- What would make a QA pass embarrassing if missed?

## Generated `SKILL.md`

Must name:

- exact dev/start command or target URL;
- critical routes/endpoints/commands;
- auth/test account setup from safe docs, or "ask user";
- evidence directory convention;
- pass/fail report format;
- independent verifier lane before a PASS verdict;
- residual-risk section.

Required acceptance block:

```markdown
## Completion Gate
- Exact end-user behavior verified: user or operator behavior the generated skill must probe.
- Value proposition exercised: specific promised outcome the QA walk covered.
- Persona outcome observed: persona-specific success or failure observed in the run.
- Evidence that proves it: screenshot, transcript, test output, or artifact path proving the observation.
- Exact command/path/route exercised: command, URL, route, file path, or tool call used.
- Repo-fit check: live repo convention or local skill contract followed.
- Acceptance mutation / hardening: mutation/hardening run, recommendation, or waiver reason.
- Residual risk: unverified path, uncovered persona, or none with reason.
```

## Eval Seed

Case prompt: "Run QA for a small change touching [surface]."

Grader passes when output includes:

- exact command/URL/surface;
- live evidence path;
- pass/fail verdict;
- residual unverified paths;
- no generic app-type checklist unrelated to this repo.
- required completion gate present.
