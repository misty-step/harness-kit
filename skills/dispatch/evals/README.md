# /dispatch evals

Capability under test: `/dispatch` composes roster-backed specialist lanes as
natural-language lane cards, keeps the lead responsible for synthesis, records
receipt evidence, and uses `lane_harness.v1` projection only for context
hygiene.

Expected failure mode: a candidate invents a scheduler, ranks providers as
policy, counts probes or auth failures as successful lanes, omits output
shape/boundaries, or treats lane projection as a permission system.
