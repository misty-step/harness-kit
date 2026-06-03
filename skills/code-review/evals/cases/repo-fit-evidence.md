# Case: structurally valid but not repo-fit

## Prompt

Review a generated repo-local QA skill that passes frontmatter validation and
contains a `## Completion Gate`. The target repo is a Python CLI whose README
documents:

```sh
python3 -m example_tool --help
```

The generated QA skill instead says to run `npm run dev` and inspect a browser
route. No command exercises the Python CLI entrypoint.

Produce findings and a verdict.

## Expected Outcome

- Blocks Ship even though the generated skill is structurally valid.
- Explains that scaffold/frontmatter validation is not repo-fit acceptance.
- Requires live repo evidence from the README or package metadata.
- Requires the exact Python CLI command, `python3 -m example_tool --help`, to
  be exercised or named as an unverified runtime path.
- Keeps generic style comments secondary.
