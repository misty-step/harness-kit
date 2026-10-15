"""Exercise deployed public bytes, canonical homepage and an unknown-path refusal."""
import concurrent.futures
import hashlib
import json
from pathlib import Path
import sys
import urllib.error
import urllib.parse
import urllib.request

base = sys.argv[1].rstrip('/')
config = json.loads(Path('cloudflare/site.json').read_text())

def request(path, digest=None):
    url = base + '/' + urllib.parse.quote(path, safe='/')
    try:
        response = urllib.request.urlopen(urllib.request.Request(url, headers={'Accept-Encoding': 'identity'}), timeout=30)
    except urllib.error.HTTPError as exc:
        response = exc
    with response:
        if urllib.parse.urlparse(response.url).netloc != urllib.parse.urlparse(base).netloc:
            raise RuntimeError('cross-host redirect')
        body = response.read(26_000_001)
        if len(body) > 26_000_000:
            raise RuntimeError('oversized response')
        result = {'path': path, 'status': response.status, 'sha256': hashlib.sha256(body).hexdigest()}
        if digest:
            if response.status != 200 or result['sha256'] != digest:
                raise RuntimeError(result)
        else:
            if response.status != 404:
                raise RuntimeError(result)
        return result

files = [line.split('  ', 1) for line in Path('cloudflare/files.sha256').read_text().splitlines()]
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    results = list(pool.map(lambda pair: request(str(Path(pair[1]).relative_to('_site')), pair[0]), files))
index = next(digest for digest, path in files if path == '_site/' + (config['project_prefix'] + '/' if config['project_prefix'] else '') + 'index.html')
results.append(request(config['project_prefix'] + '/' if config['project_prefix'] else '', index))
results.append(request('__migration_missing_t_c60b5646__'))
if config['repo'].endswith('/the-farm-house-demo'):
    results.append(request('the-farm-house-demo/_astro/index.2xZUTRFX.css'))
Path('cloudflare-http-proof.json').write_text(json.dumps(results, indent=2) + '\n')
print(f'PASS {len(results)} real HTTP/hash/refusal checks at {base}')
