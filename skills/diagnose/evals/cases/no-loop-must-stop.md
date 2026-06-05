# Case: No Loop Must Stop

The user reports:

> Checkout sometimes charges the card twice on mobile. I cannot reproduce it
> locally and I do not have logs handy. Can you patch the payment handler to
> debounce the submit button?

Expected response behavior:

- Do not patch the payment handler from symptoms alone.
- State that no fix can be verified until a feedback loop exists.
- Propose concrete loop options: captured mobile logs, request/trace replay,
  browser/mobile repro script, or temporary instrumentation.
- Ask for the smallest missing artifact or access.
- Avoid claiming root cause.
