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
import importlib.util

# Ensure we can find aktools when bundled
if getattr(sys, 'frozen', False):
    # Running in a PyInstaller bundle
    bundle_dir = sys._MEIPASS
else:
    # Running in a normal Python environment
    bundle_dir = os.path.dirname(os.path.abspath(__file__))


def main():
    parser = argparse.ArgumentParser(description='AKTools Server')
    parser.add_argument('--port', type=int, default=8080, help='Port to listen on')
    parser.add_argument('--host', type=str, default='127.0.0.1', help='Host to bind to')
    args = parser.parse_args()

    # Find aktools location
    try:
        import aktools
        aktools_dir = os.path.dirname(aktools.__file__)
    except ImportError as e:
        print(f"Error: Failed to import aktools: {e}", file=sys.stderr)
        sys.exit(1)

    # Import main app from aktools directory
    try:
        import uvicorn

        # Change to aktools directory so uvicorn can find main:app
        original_dir = os.getcwd()
        os.chdir(aktools_dir)

        # Import main module from aktools directory
        spec = importlib.util.spec_from_file_location("main", os.path.join(aktools_dir, "main.py"))
        main_module = importlib.util.module_from_spec(spec)
        sys.modules["main"] = main_module
        spec.loader.exec_module(main_module)
        app = main_module.app

        # Change back to original directory
        os.chdir(original_dir)
    except ImportError as e:
        print(f"Error: Failed to import: {e}", file=sys.stderr)
        sys.exit(1)

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
