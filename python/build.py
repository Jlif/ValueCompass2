#!/usr/bin/env python3
"""
AKTools PyInstaller Build Script
================================
Builds a standalone executable for AKTools that can be bundled with Tauri.

Usage:
    python build.py                    # Build for current platform
    python build.py --output-dir=../src-tauri/bin  # Custom output
    python build.py --onefile          # Build single executable (slower startup)
    python build.py --onedir           # Build directory (faster startup, default)

Requirements:
    pip install -r requirements.txt

Output:
    dist/aktools              - Linux/macOS executable
    dist/aktools.exe          - Windows executable
    dist/aktools/             - Directory mode output (if --onedir)
"""

import os
import sys
import shutil
import argparse
import subprocess
import platform
from pathlib import Path


def get_platform():
    """Get current platform identifier."""
    system = platform.system().lower()
    if system == 'darwin':
        return 'macos'
    return system


def run_pyinstaller(output_dir: str, onefile: bool = False, clean: bool = False):
    """Run PyInstaller to build the executable."""

    script_dir = Path(__file__).parent.absolute()
    entry_point = script_dir / 'aktools_server.py'

    if not entry_point.exists():
        print(f"Error: Entry point not found: {entry_point}")
        sys.exit(1)

    # Build command
    cmd = [
        sys.executable, '-m', 'PyInstaller',
        str(entry_point),
        '--name=aktools',
        '--clean' if clean else '',
        '--noconfirm',
    ]

    if onefile:
        cmd.append('--onefile')
    else:
        cmd.append('--onedir')

    # Add hidden imports for AKTools
    hidden_imports = [
        'aktools',
        'aktools.api',
        'akshare',
        'uvicorn',
        'uvicorn.logging',
        'uvicorn.loops',
        'uvicorn.loops.auto',
        'uvicorn.protocols',
        'uvicorn.protocols.http',
        'uvicorn.protocols.http.auto',
        'uvicorn.lifespan',
        'uvicorn.lifespan.on',
        'fastapi',
        'fastapi.middleware',
        'fastapi.middleware.cors',
        'pandas',
        'numpy',
        'requests',
        'urllib3',
        'certifi',
        'charset_normalizer',
        'idna',
        'pydantic',
        'starlette',
        'typing_extensions',
        'anyio',
        'sniffio',
    ]

    for imp in hidden_imports:
        cmd.extend(['--hidden-import', imp])

    # Collect all data files from aktools and akshare
    cmd.extend([
        '--collect-all', 'aktools',
        '--collect-all', 'akshare',
        '--collect-all', 'pandas',
    ])

    # Strip to reduce size (optional, removes debug symbols)
    # cmd.append('--strip')

    # Remove empty strings from command
    cmd = [c for c in cmd if c]

    print(f"Running: {' '.join(cmd)}")
    print(f"Working directory: {script_dir}")

    result = subprocess.run(cmd, cwd=script_dir)

    if result.returncode != 0:
        print("Error: PyInstaller failed")
        sys.exit(1)

    # Copy to output directory
    dist_dir = script_dir / 'dist'
    output_path = Path(output_dir).absolute()

    if output_path.exists():
        print(f"Cleaning existing output: {output_path}")
        shutil.rmtree(output_path)

    if onefile:
        # Single file mode
        executable_name = 'aktools.exe' if platform.system() == 'Windows' else 'aktools'
        source = dist_dir / executable_name
        output_path.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, output_path / executable_name)
        print(f"Built: {output_path / executable_name}")
    else:
        # Directory mode
        source = dist_dir / 'aktools'
        shutil.copytree(source, output_path)
        print(f"Built: {output_path}")

    # Print size info
    if output_path.exists():
        if output_path.is_dir():
            total_size = sum(f.stat().st_size for f in output_path.rglob('*') if f.is_file())
        else:
            total_size = output_path.stat().st_size
        print(f"Total size: {total_size / (1024*1024):.1f} MB")


def main():
    parser = argparse.ArgumentParser(description='Build AKTools standalone executable')
    parser.add_argument(
        '--output-dir', '-o',
        default='dist',
        help='Output directory for the built executable (default: dist)'
    )
    parser.add_argument(
        '--onefile',
        action='store_true',
        help='Build single executable file (slower startup, smaller size)'
    )
    parser.add_argument(
        '--onedir',
        action='store_true',
        default=True,
        help='Build directory (faster startup, larger size, default)'
    )
    parser.add_argument(
        '--clean',
        action='store_true',
        help='Clean PyInstaller cache before building'
    )
    args = parser.parse_args()

    # Default output location for Tauri integration
    default_output = Path(__file__).parent.parent / 'src-tauri' / 'bin' / get_platform()

    output_dir = args.output_dir
    if output_dir == 'dist':
        output_dir = str(default_output)

    print(f"Building AKTools for {get_platform()}...")
    print(f"Output directory: {output_dir}")
    print(f"Mode: {'single file' if args.onefile else 'directory'}")
    print()

    run_pyinstaller(output_dir, onefile=args.onefile, clean=args.clean)

    print()
    print("Build complete!")
    print()
    print("Next steps:")
    print("1. Verify the executable works: ./aktools --port=8080")
    print("2. Tauri will automatically find the executable in src-tauri/bin/")
    print("3. Update tauri.conf.json sidecar configuration if needed")


if __name__ == '__main__':
    main()
