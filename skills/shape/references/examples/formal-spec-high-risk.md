# Context Packet: Payment permission rewrite

## Goal

Only account owners can approve refunds above the configured threshold.

## Oracle (Definition of Done)

- [ ] `pytest tests/acceptance/test_refund_permissions.py` exits 0.
- [ ] `pytest tests/payments/test_refund_rules.py` exits 0.

## Formal Spec

- Formal Spec Required: yes (triggers: money/security behavior, example-driven behavior, expensive manual regression)
- Informal spec: Refund approvals above the threshold must reject non-owners and allow the account owner with an audit entry.
- Formal examples: hypothetical `tests/fixtures/refund-permission-scenarios.feature` covers owner approval, non-owner rejection, threshold boundary, and audit logging.
- Acceptance oracle: `pytest tests/acceptance/test_refund_permissions.py` must fail before implementation and pass after.
- Hardening budget: `/hardening risk` on changed payment modules, then bounded `/hardening mutation` for permission branches and `/hardening acceptance` for the scenario fixture; cap at 90 minutes unless a survivor is blocking.
- Waiver path: Delivery lead may waive property testing only after a fresh critic confirms no useful invariant beyond the acceptance examples; waiver records residual risk in the `/deliver` receipt.

## Implementation Sequence

1. Add failing acceptance coverage from the scenario fixture.
2. Add focused unit tests for the threshold and owner checks.
3. Implement the permission rule and audit entry.
4. Run the formal-spec ladder evidence named above.
