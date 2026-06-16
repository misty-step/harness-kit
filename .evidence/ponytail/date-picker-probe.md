# Ponytail Date Picker Probe

Backlog: `106`
Date: 2026-06-16

Synced skill path: `skills/.external/dietrich-ponytail/SKILL.md`

Probe prompt: "Add a date picker."

Evidence from the synced skill:

- The ladder asks whether the feature needs to exist, then checks stdlib, then
  native platform features before dependencies.
- The native-platform example is exactly `<input type="date">` over a picker
  library.
- The default `full` mode enforces stdlib and native-first behavior.

Expected Ponytail response for this probe: choose native `<input type="date">`
unless the user names requirements that the native control cannot meet.
