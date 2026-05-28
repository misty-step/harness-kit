# /harness-engineering sync

Pull primitives from harness-kit into project-local harness directories.

## How it works

Reads `.harness-kit.yaml`, pulls declared skills/agents from GitHub into
project-local harness directories. When a local harness-kit checkout exists,
uses symlinks instead (edits propagate instantly).

## Marker file convention

Managed primitives have a `.harness-kit` marker file.
/harness-engineering sync only touches directories with this marker.
