# The Application Floor

Operator standing directive (2026-07-05). Every application Misty Step
builds ships with all of these, under any circumstances. This is the floor,
not the ceiling; grooming sessions treat a missing item as a backlog gap,
and `/shape` packets for new products include the floor items in scope or
name an explicit waiver per item.

## The floor

1. **Marketing site.** Branded, deployed publicly. Misty Step aesthetic via
   the shared site kit, per-repo `DESIGN.md` brand identity, strong pitch,
   real screenshots/GIF walkthroughs, user-facing release notes (Landmark),
   footer link contract (GitHub repo when public, mistystep.io always,
   Weave for weave-family products). Program epic: powder `misty-step-910`;
   kit: `aesthetic-907`. Evidence bar per `/showcase`: no public claim
   without a screenshot, command, or demo path behind it.
2. **The five faces.** One core, every face (operator ratification,
   2026-07-04: "every single application needs its core functionality,
   needs the API, and unless there is a strong reason for an exception,
   they should pretty much all also have an MCP and a CLI and a skill that
   they ship and a UI"). Concretely: **API + CLI + MCP server + shipped
   skill + UI** over one core; SDK where external consumers exist. A face
   is complete only if it covers the core verbs — an MCP that can read but
   not write (powder, 2026-07-04: no `create_card`, groom fell back to raw
   HTTP) is a floor violation, not a partial credit. Exceptions are named
   waivers per face, never silent omissions.
3. **Documentation.** An operator can go zero-to-productive from the repo
   alone: README with a real quickstart, an operator walkthrough for
   anything with a UI or serve mode, honest help text.
4. **CI and quality gates.** The repo gate runs in CI, gates the diff, and
   is never weakened to get green (`quality-gates.md`).
5. **Relative infrastructure agnosticism.** No load-bearing coupling to one
   host. Fly/Sanctum/Pages are deploy targets, not architecture.
6. **Deep modularity.** Ousterhout: interfaces far simpler than
   implementations; no shallow pass-throughs or speculative abstraction.
7. **Test coverage approaching 100%, spanning unit, integration, and
   end-to-end.** Coverage earns its number through behavior-asserting
   tests (`verification-system-first.md`), not implementation mirrors.
8. **Rust — or the strongest static typing the platform boundary allows.**
   Maximize compile-time correctness guarantees. Every non-Rust surface
   names its constraint.
9. **Frictionless onboarding.** Push-button wherever possible: one
   click-to-copy command from zero to fully working — including daemons,
   agents, and indicators actually *running*, not just installed. Where
   self-hosting is the design (e.g. Canary), containerize it, document it,
   and ship agent-ready setup prompts. A `doctor` command that fails loudly
   when the deployment is dead is part of onboarding, not polish.

## The case study that made this doctrine

Counterspell, 2026-07-05: the tool existed, was installed, configured, and
green — and still failed the operator, because `setup` installed only the
annotation agent (the armed watcher had no daemon), the installed binary
was two days stale, and the menu-bar indicator's host app was never
installed. Three "done" claims, zero live protection. Floor items 9 and 3
exist so "installed" can never again masquerade as "running": onboarding
ends at verified-live, and doctor is the proof.
