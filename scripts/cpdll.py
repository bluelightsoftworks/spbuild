#!/usr/bin/env python3

import subprocess
import shutil
from pathlib import Path

def find_objdump():
    """Find the objdump executable."""
    variants = [
        'x86_64-w64-mingw32-objdump',
        'objdump'
    ]

    for cmd in variants:
        if shutil.which(cmd):
            return cmd

    return None

def get_dll_dependencies(file_path, objdump_cmd):
    """Get DLL dependencies using objdump."""
    try:
        result = subprocess.run(
            [objdump_cmd, '-p', str(file_path)],
            capture_output=True,
            text=True,
            check=True
        )
        # Parse objdump output for DLL Name entries
        deps = []
        for line in result.stdout.split('\n'):
            if 'DLL Name:' in line:
                dll = line.split('DLL Name:')[1].strip()
                deps.append(dll)
        return deps
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []

def find_dll(dll_name, search_paths):
    """Find a DLL in the given search paths."""
    for search_path in search_paths:
        search_path = Path(search_path)
        if not search_path.exists():
            continue
        for dll_path in search_path.rglob(dll_name):
            if dll_path.is_file():
                return dll_path
    return None

def is_system_dll(dll_name):
    """Check if DLL is a Windows system DLL that shouldn't be copied."""
    dll_lower = dll_name.lower()
    system_dlls = [
        'kernel32.dll', 'user32.dll', 'advapi32.dll', 'msvcrt.dll',
        'gdi32.dll', 'shell32.dll', 'ole32.dll', 'oleaut32.dll',
        'ws2_32.dll', 'ntdll.dll', 'comctl32.dll'
    ]
    # Skip api-ms-win-* DLLs (Windows API sets)
    if dll_lower.startswith('api-ms-win-'):
        return True
    return dll_lower in system_dlls

def copy_dependencies(exe_path, output_dir, search_paths=None):
    """Recursively copy all DLL dependencies."""
    # Find objdump
    objdump_cmd = find_objdump()
    if objdump_cmd is None:
        print("Error: objdump not found!")
        print("Install with: sudo apt install mingw-w64-tools")
        return

    print(f"Using: {objdump_cmd}")

    if search_paths is None:
        search_paths = [
            '/usr/x86_64-w64-mingw32',
            '/usr/lib/gcc/x86_64-w64-mingw32',
        ]

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    processed = set()
    to_process = [Path(exe_path)]

    while to_process:
        current_file = to_process.pop(0)
        current_name = current_file.name

        # Skip if we've already processed this file
        if current_name in processed:
            continue

        processed.add(current_name)

        print(f"\nAnalyzing: {current_name}")

        # Get dependencies of current file
        deps = get_dll_dependencies(current_file, objdump_cmd)

        if deps:
            print(f"  Found {len(deps)} dependencies:")

        for dll in deps:
            # Skip if is system DLL
            if is_system_dll(dll):
                print(f"    WARN - {dll} is a system DLL, skipping")
                continue

            # Check if already in output directory
            dest = output_dir / dll
            if dest.exists():
                print(f"    OK - {dll} already copied")
                # Still need to process its dependencies if we haven't
                if dll not in processed:
                    to_process.append(dest)
                continue

            # Find the DLL in search paths
            dll_path = find_dll(dll, search_paths)

            if dll_path is None:
                print(f"    WARN - {dll} not found")
                continue

            # Copy to output directory
            print(f"    -> {dll} copying from {dll_path.parent.name}/")
            shutil.copy2(dll_path, dest)

            # Add this DLL to processing queue to check ITS dependencies
            to_process.append(dest)

if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: cpdll.py <path/to/executable.exe>")
        sys.exit(1)

    exe_path = Path(sys.argv[1])

    if not exe_path.exists():
        print(f"Error: {exe_path} not found")
        sys.exit(1)

    output_dir = exe_path.parent

    print("=" * 60)
    print(f"Executable: {exe_path}")
    print(f"Output directory: {output_dir}")
    print("=" * 60)

    copy_dependencies(exe_path, output_dir)

    print("\n" + "=" * 60)
    print("Done!")