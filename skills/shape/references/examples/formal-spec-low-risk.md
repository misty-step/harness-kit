# Context Packet: Typo fix in setup copy

## Goal

Correct the misspelled setup command label in the docs copy.

## Oracle (Definition of Done)

- [ ] `cargo run --locked -p harness-kit-checks -- check-docs-site --self-test` exits 0.

## Risk Notes

Formal Spec Required: no. This is a single-file wording correction, does not
change rules or contracts, has an existing docs drift oracle, and a regression
is cheap to detect.

## Implementation Sequence

single chunk
