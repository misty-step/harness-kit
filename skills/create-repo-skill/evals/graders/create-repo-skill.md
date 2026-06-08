# Create Repo Skill Grader

Run the artifact validator on the generated skill directory:

```sh
cargo run --locked -p harness-kit-checks -- repo-skill validate <target-repo>/.agents/skills/<name>
cargo run --locked -p harness-kit-checks -- eval-grader create-repo-skill <target-repo>/.agents/skills/<name>
```

The grader must inspect generated files on disk. Transcript-only grading is a
fallback for legacy eval fixtures, not acceptance for real repo skill output.
