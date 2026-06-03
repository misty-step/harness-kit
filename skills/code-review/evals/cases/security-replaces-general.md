# Case: Security Replaces General Lens

Changed files:

- `src/auth/login.ts`
- `tests/auth.spec.ts`

Expected internal bench:

- includes `critic`;
- includes `security` for the auth path;
- excludes `grug` because the security rule replaces that general lens;
- includes `cooper` for the test path;
- never exceeds 5 reviewers.
