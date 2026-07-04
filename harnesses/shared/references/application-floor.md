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
   For any surface that ships HTML/JS/CSS, "end-to-end" means a real
   engine executes the artifact — three tiers, all mandatory: (a) every
   embedded or generated artifact is syntax-validated in the gate
   (extract inline scripts → parse them; templates fail closed on empty
   interpolation or missing assets — a build error, never a placeholder);
   (b) a smoke load of each major page in a headless browser asserts
   zero console errors; (c) the few golden user paths are exercised
   behaviorally (click → visible state change) at desktop AND ~390px
   widths. Substring assertions against rendered HTML do not count as
   coverage of the code inside it: they test the transcript, not the
   program.
8. **Rust — or the strongest static typing the platform boundary allows.**
   Maximize compile-time correctness guarantees. Every non-Rust surface
   names its constraint.
9. **Frictionless onboarding.** Push-button wherever possible: one
   click-to-copy command from zero to fully working — including daemons,
   agents, and indicators actually *running*, not just installed. Where
   self-hosting is the design (e.g. Canary), containerize it, document it,
   and ship agent-ready setup prompts. A `doctor` command that fails loudly
   when the deployment is dead is part of onboarding, not polish.

## The case studies that made this doctrine

Counterspell, 2026-07-05: the tool existed, was installed, configured, and
green — and still failed the operator, because `setup` installed only the
annotation agent (the armed watcher had no daemon), the installed binary
was two days stale, and the menu-bar indicator's host app was never
installed. Three "done" claims, zero live protection. Floor items 9 and 3
exist so "installed" can never again masquerade as "running": onboarding
ends at verified-live, and doctor is the proof.

Sanctum artifacts, 2026-07-05: one Rust raw-string escaping slip
(`main.rs`, a `\"` shipped verbatim into inline JS) made the page's entire
script throw a SyntaxError on load — star toggle, search, and pagination
all dead for every user — while 84 tests stayed green, because every one
of them substring-matched the rendered HTML and none parsed or executed
the JavaScript. Two independent verification lanes then confirmed the
server behavior and still missed it, because both stopped at the layer
boundary they owned. Floor item 7's real-engine tiers exist so that a
page that cannot even parse can never again ship green: syntax-gate the
embedded artifact, smoke-load for console errors, and click the golden
path in a real browser — one verifier always stands at the user boundary.
