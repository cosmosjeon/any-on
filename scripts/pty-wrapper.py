#!/usr/bin/env python3
"""PTY wrapper for running commands with proper stdin/stdout handling"""
import sys
import pty
import subprocess

def main():
    if len(sys.argv) < 2:
        print("Usage: pty-wrapper.py <command> [args...]", file=sys.stderr)
        sys.exit(1)

    # Run command in PTY
    try:
        pty.spawn(sys.argv[1:])
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
