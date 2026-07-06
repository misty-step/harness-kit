#!/usr/bin/env python3
"""artifact_serve — minimal static server for the artifacts root.

Hermes-independent replacement for hermes_artifact_server.py. Serves
~/artifacts/public on 127.0.0.1:<port>; Tailscale `serve` maps
https://<host>.ts.net/artifacts -> this. Zero LLM tokens; stdlib only.
Directory requests resolve to index.html.

Also carries one scoped relay route, /api/bridge-answer, so the Bridge
page (~/.factory-lanes/scripts/bridge.py) can let the operator answer a
NEEDS YOU question from a text box instead of a copy-pasted curl command.
The relay forwards to exactly one upstream shape -- POST a powder run
answer -- using a key read server-side; it is not a general proxy.
/api/bridge-refresh re-runs bridge.py so the page reflects the answer.
"""
import argparse
import functools
import json
import os
import subprocess
import urllib.error
import urllib.request
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler

HOME = os.path.expanduser("~")
BRIDGE_KEY_PATH = os.path.join(HOME, ".factory-lanes", ".powder-bridge-key")
BRIDGE_SCRIPT = os.path.join(HOME, ".factory-lanes", "scripts", "bridge.py")
POWDER_BASE = "https://bastion.tail5f5eb4.ts.net:10001"


class Handler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        # The Bridge page is mirrored on other tailnet hosts (Sanctum/bastion)
        # but the answer relay lives only here; cross-origin POSTs are
        # tailnet-private, so a permissive origin is acceptable.
        self.send_header("Access-Control-Allow-Origin", "*")
        super().end_headers()

    def log_message(self, *args):  # quiet
        pass

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Access-Control-Max-Age", "86400")
        self.end_headers()

    def do_POST(self):
        if self.path == "/api/bridge-answer":
            return self._bridge_answer()
        self.send_error(404)

    def do_GET(self):
        if self.path == "/api/bridge-refresh":
            return self._bridge_refresh()
        if self.path.startswith("/api/bridge-retro"):
            return self._bridge_retro()
        return super().do_GET()

    def _bridge_retro(self):
        """Operator run-handle for time-based activity reports (operator ask
        2026-07-06: 'no idea how to see or run time based activity reports').
        Kicks weave-fleet-retro DETACHED (runs ~1-2 min: repo sweep + powder
        + render + shelf publish + feed post) and returns immediately with
        where the result lands. Windows: daily | weekly | custom (since/until
        RFC3339 or YYYY-MM-DD). The finished report self-announces via the
        tool's own feed post, so the deck's feed shows it when ready."""
        from urllib.parse import urlparse, parse_qs
        q = parse_qs(urlparse(self.path).query)
        window = (q.get("window", ["daily"])[0] or "daily").strip()
        if window not in ("daily", "weekly", "custom"):
            return self._json(400, {"error": "window must be daily|weekly|custom"})
        # Resolve the synthesis key at run time (keychain -> op service account
        # -> op read); launchd context has no secrets env, and ~/.secrets does
        # not carry OPENROUTER_API_KEY. Nothing is persisted; fleet-retro still
        # fails open to tables+banner if resolution fails.
        retro_args = ["--window", window]
        cmd = ["/bin/zsh", "-c",
               'export OP_SERVICE_ACCOUNT_TOKEN="${OP_SERVICE_ACCOUNT_TOKEN:-$('
               'security find-generic-password -a "$USER" -s op-agent -w 2>/dev/null)}"; '
               'export OPENROUTER_API_KEY="${OPENROUTER_API_KEY:-$('
               'op read "op://Agents/OPENROUTER_API_KEY/credential" 2>/dev/null)}"; '
               'source ~/.secrets 2>/dev/null; '
               'exec /Users/phaedrus/.cargo/bin/cargo run -q -p weave-fleet-retro -- "$@"',
               "retro"] + retro_args
        if window == "custom":
            since = (q.get("since", [""])[0] or "").strip()
            if not since:
                return self._json(400, {"error": "custom window requires since="})
            if len(since) == 10:
                since += "T00:00:00Z"
            cmd += ["--since", since]
            until = (q.get("until", [""])[0] or "").strip()
            if until:
                if len(until) == 10:
                    until += "T23:59:59Z"
                cmd += ["--until", until]
        try:
            subprocess.Popen(
                cmd, cwd=os.path.expanduser("~/Development/weave"),
                stdout=open("/tmp/bridge-retro.log", "ab"),
                stderr=subprocess.STDOUT,
                start_new_session=True)
        except OSError as err:
            return self._json(500, {"error": str(err)})
        base = "https://bastion.tail5f5eb4.ts.net/artifacts/a/fleet-retro"
        url = {"daily": f"{base}/daily/index.html",
               "weekly": f"{base}/weekly/index.html"}.get(window)
        return self._json(202, {
            "status": "generating",
            "eta": "~2 min",
            "url": url,
            "note": "the finished report also announces itself on the feed",
        })

    def _bridge_answer(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        try:
            body = json.loads(self.rfile.read(length) or b"{}")
        except json.JSONDecodeError:
            return self._json(400, {"error": "invalid json"})
        run_id = str(body.get("run_id") or "").strip()
        answer = str(body.get("answer") or "").strip()
        actor = str(body.get("actor") or "operator").strip()
        if not run_id or not answer:
            return self._json(400, {"error": "run_id and answer are required"})
        try:
            key = open(BRIDGE_KEY_PATH).read().strip()
        except OSError as err:
            return self._json(500, {"error": f"no bridge key: {err}"})
        req = urllib.request.Request(
            f"{POWDER_BASE}/api/v1/runs/{run_id}/answer",
            data=json.dumps({"actor": actor, "answer": answer}).encode(),
            method="POST",
        )
        req.add_header("Authorization", f"Bearer {key}")
        req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req, timeout=8) as resp:
                return self._json(resp.status, json.loads(resp.read()))
        except urllib.error.HTTPError as err:
            return self._json(err.code, json.loads(err.read() or b"{}"))
        except (urllib.error.URLError, TimeoutError) as err:
            return self._json(502, {"error": f"powder unreachable: {err}"})

    def _bridge_refresh(self):
        r = subprocess.run(
            ["python3", BRIDGE_SCRIPT], capture_output=True, text=True, timeout=30
        )
        if r.returncode != 0:
            return self._json(500, {"error": (r.stderr or r.stdout).strip()[:500]})
        return self._json(200, {"ok": True})

    def _json(self, status, payload):
        data = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=8789)
    ap.add_argument("--root", default=os.path.expanduser("~/artifacts/public"))
    a = ap.parse_args()
    os.makedirs(a.root, exist_ok=True)
    handler = functools.partial(Handler, directory=a.root)
    httpd = ThreadingHTTPServer((a.host, a.port), handler)
    print(f"artifact_serve: {a.host}:{a.port} -> {a.root}")
    httpd.serve_forever()


if __name__ == "__main__":
    main()
