"""Stage only the exact, reviewed public Pages artifact; refuse changed bytes."""
import hashlib
import json
from pathlib import Path
import shutil
import sys

config = json.loads(Path('cloudflare/site.json').read_text())
source = Path(sys.argv[1])
root = Path('_site')
if root.exists():
    raise RuntimeError('refusing stale staging directory')
root.mkdir()
for line in Path('cloudflare/files.sha256').read_text().splitlines():
    digest, path = line.split('  ', 1)
    relative = Path(path).relative_to('_site')
    local = relative.relative_to(config['project_prefix']) if config['project_prefix'] else relative
    if local.is_absolute() or '..' in local.parts:
        raise ValueError('invalid asset path')
    origin = source / local
    body = origin.read_bytes()
    if hashlib.sha256(body).hexdigest() != digest:
        raise RuntimeError(f'Pages byte parity failed: {local}')
    dest = root / relative
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(origin, dest)
if config['project_prefix']:
    (root / '_redirects').write_text(f'/ /{config["project_prefix"]}/ 302\n')
