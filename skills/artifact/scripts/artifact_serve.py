#!/usr/bin/env python3
"""artifact_serve — minimal static server for the artifacts root.

Hermes-independent replacement for hermes_artifact_server.py. Serves
~/artifacts/public on 127.0.0.1:<port>; Tailscale `serve` maps
https://<host>.ts.net/artifacts -> this. Zero LLM tokens; stdlib only.
Directory requests resolve to index.html.
"""
import argparse, os, functools
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler


class Handler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def log_message(self, *args):  # quiet
        pass


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
