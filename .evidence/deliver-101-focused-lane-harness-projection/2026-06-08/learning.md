# Learning Packet: 101 Focused Lane Harness Projection

## What Worked
- Treating focused harness projection as a filesystem/config projection kept the
  design simple and testable.
- Recording provider failures as typed receipts handled exhausted credits,
  auth, entitlement, timeout, spawn, and sentinel failures without hiding them
  behind automatic fallback.
- Fresh critics caught a real model-contract gap: provider target validation is
  insufficient unless `model_override` is also scoped to the selected provider's
  roster entries.

## What To Codify Later
- External skill aliases need a second slice: validation exists, but
  materialization should copy or symlink pinned external skills into the
  projected roots before claiming complete external skill support.
- Tool labels are currently a receipt/manifest contract. Cross-provider tool
  enforcement should stay adapter-specific and fail closed when unsupported.
- Conditional real-provider smokes should prove each provider honors the
  projected config roots before any provider is advertised as isolated beyond
  the fake-provider filesystem contract.

## Waivers
- Hardening/formal-spec waiver: backlog 101 did not require a formal spec, and
  the implementation is covered by manifest validation tests, fake-provider
  dispatch tests, receipt summary tests, roster fixture validation, runtime
  primitive validation, and the final Dagger gate.
