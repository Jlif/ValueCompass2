#!/usr/bin/env python3
"""
AKTools Server Wrapper
======================
A simple wrapper around AKTools to ensure proper startup and shutdown behavior.
This script is used as the entry point for PyInstaller bundling.

Usage:
    python aktools_server.py --port=8080 --host=127.0.0.1
"""

import sys
import os
import argparse
import signal

# Ensure we can find aktools when bundled
if getattr(sys, 'frozen', False):
    # Running in a PyInstaller bundle
    bundle_dir = sys._MEIPASS
else:
    # Running in a normal Python environment
    bundle_dir = os.path.dirname(os.path.abspath(__file__))

# Add bundled site-packages to path if exists
site_packages = os.path.join(bundle_dir, 'site-packages')
if os.path.exists(site_packages) and site_packages not in sys.path:
    sys.path.insert(0, site_packages)


def main():
    parser = argparse.ArgumentParser(description='AKTools Server')
    parser.add_argument('--port', type=int, default=8080, help='Port to listen on')
    parser.add_argument('--host', type=str, default='127.0.0.1', help='Host to bind to')
    args = parser.parse_args()

    # Import aktools after path setup
    try:
        from aktools import get_default_application
        import uvicorn
    except ImportError as e:
        print(f"Error: Failed to import aktools: {e}", file=sys.stderr)
        print(f"Python path: {sys.path}", file=sys.stderr)
        sys.exit(1)

    app = get_default_application()

    # Handle signals gracefully
    def signal_handler(sig, frame):
        print("\nShutting down server...", file=sys.stderr)
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # Start server
    print(f"Starting AKTools server on {args.host}:{args.port}", file=sys.stderr)
    uvicorn.run(
        app,
        host=args.host,
        port=args.port,
        log_level="warning",
        access_log=False
    )


if __name__ == '__main__':
    main()
