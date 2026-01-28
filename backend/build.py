#!/usr/bin/env python3
"""
Build script for packaging the Python backend using PyInstaller.
Creates a standalone executable that can be bundled with Electron.
"""

import subprocess
import sys
import platform
from pathlib import Path

def build_backend():
    """Build the FastAPI backend into a standalone executable."""
    
    # Determine the output name based on platform
    system = platform.system().lower()
    if system == "windows":
        output_name = "any2bibtex-backend.exe"
    else:
        output_name = "any2bibtex-backend"
    
    # PyInstaller command
    cmd = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",           # Single executable
        "--clean",             # Clean build
        "--noconfirm",         # Overwrite without asking
        "--name", "any2bibtex-backend",
        "--distpath", "../resources/backend",  # Output to resources folder
        "--workpath", "./build",
        "--specpath", "./build",
        # Hidden imports for FastAPI/Uvicorn
        "--hidden-import", "uvicorn.logging",
        "--hidden-import", "uvicorn.loops",
        "--hidden-import", "uvicorn.loops.auto",
        "--hidden-import", "uvicorn.protocols",
        "--hidden-import", "uvicorn.protocols.http",
        "--hidden-import", "uvicorn.protocols.http.auto",
        "--hidden-import", "uvicorn.protocols.websockets",
        "--hidden-import", "uvicorn.protocols.websockets.auto",
        "--hidden-import", "uvicorn.lifespan",
        "--hidden-import", "uvicorn.lifespan.on",
        "--hidden-import", "email.mime.text",
        "main.py"
    ]
    
    print(f"Building backend for {system}...")
    print(f"Command: {' '.join(cmd)}")
    
    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    
    if result.returncode == 0:
        print(f"\n✅ Build successful! Output: resources/backend/{output_name}")
    else:
        print(f"\n❌ Build failed with exit code {result.returncode}")
        sys.exit(result.returncode)

if __name__ == "__main__":
    build_backend()
