#!/usr/bin/env python3
"""Serve the docs site locally with correct WASM MIME type.

Usage: python3 serve_docs.py
Then open http://localhost:8081
"""
import http.server
import os

class Handler(http.server.SimpleHTTPRequestHandler):
    def guess_type(self, path):
        if path is not None:
            if path.endswith('.wasm'):
                return 'application/wasm'
            if path.endswith('.js'):
                return 'application/javascript'
        return super().guess_type(path)

port = 8081
print(f"Open http://localhost:{port}")
print(f"Playground: http://localhost:{port}/playground.html")
http.server.HTTPServer(('', port), Handler).serve_forever()
