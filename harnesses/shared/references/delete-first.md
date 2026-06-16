# Delete-First Lens

Use while shaping, refactoring, or automating. The order matters.

1. **Question the requirement.** What user outcome disappears if this is not
   built? Who owns the requirement?
2. **Delete.** Can the feature, file, process step, dependency, mode, or
   abstraction be removed entirely?
3. **Simplify.** Can stdlib, native platform behavior, existing repo code, or a
   manual step cover the need?
4. **Speed up.** Optimize only after the thing survives deletion and
   simplification.
5. **Automate.** Automate only repeated, verified, bounded work.

## Boundaries

Do not delete explicit user requirements, trust-boundary validation, data-loss
prevention, security measures, accessibility basics, or acceptance evidence.

## Prompt

Before adding or automating, answer in three lines:

- Requirement questioned:
- Deleted or simplified:
- Only then optimized/automated because:

If the third line is doing all the work, the design is probably backwards.
