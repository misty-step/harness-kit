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
- Exact end-user behavior verified:
- Value proposition exercised:
- Persona outcome observed:
- Live evidence that proves it:
- Exact command/path/route exercised:
- Repo-fit check:
- Acceptance mutation / hardening:
- Residual unverified paths:
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
