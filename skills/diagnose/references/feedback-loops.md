# Feedback Loops

The first debugging deliverable is a loop that can prove the symptom is present
and later prove it is gone. Pick the smallest loop that exercises the real
failure, then improve the loop before changing production code.

## Loop Menu

Prefer loops in this order when they fit the symptom:

1. **Failing test** at the highest seam that reaches the bug.
2. **HTTP/API script** such as `curl` or a checked-in request fixture.
3. **Local third-party API emulator** such as
   `npx emulate --service github,stripe` plus request replay when `emulate.dev`
   supports the provider; it exercises SDK/protocol seams without network.
4. **CLI fixture** that runs one command and diffs stdout, stderr, exit code,
   or a generated artifact.
5. **Browser script** for UI behavior, asserting DOM, console, network, or
   visible output.
6. **Trace replay** from a captured request, payload, event stream, HAR file,
   log bundle, or production-safe fixture.
7. **Throwaway harness** that starts the smallest runnable subset of the
   system and calls the failing path.
8. **Property or fuzz loop** when the bug is broad-domain, intermittent, or
   input-sensitive.
9. **Bisect harness** for regressions between known-good and known-bad states.
10. **Differential loop** that compares old vs new version, config A vs config
   B, or two implementations on the same input.
11. **HITL loop** only when a person must reproduce it. Add targeted logs,
    give exact steps, read the captured output, then refine.

## Loop Quality

Before moving to root-cause hypotheses, improve the loop:

- **Faster:** remove unrelated setup, cache heavy initialization, narrow scope.
- **Sharper:** assert the specific symptom, not "did not crash."
- **More deterministic:** pin time, seed randomness, isolate filesystem and
  network, freeze external data where possible.
- **Closer to the real path:** avoid shallow seams that cannot reproduce the
  caller behavior that failed.

For nondeterministic bugs, raise the reproduction rate. A 50 percent flake is
debuggable; a 1 percent flake usually needs stress, parallel repetition,
timing injection, or better instrumentation before investigation can proceed.

## Correct Seam Rule

A regression test earns trust only when it exercises the bug pattern as it
appeared. If the only available test seam is too shallow, document that as the
finding instead of writing a false-confidence test. Fix the bug when the root
cause is proven, then fix the missing seam directly or file a ticket with a
focused `/critique --lens ousterhout` follow-up.

## No-Loop Stop

If no believable loop can be built with current access, stop and report:

- what loops were attempted;
- the missing artifact or access;
- the smallest useful request to the user or operator;
- any temporary instrumentation needed and how it will be removed.

Do not proceed to speculative fixes without a loop.
