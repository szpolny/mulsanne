#!/usr/bin/env python3
"""Run the Windows bridge beside an already-running Steam/Proton LMU process.

Usage: python3 scripts/run-lmu-bridge.py /path/to/lmu-bridge.exe [Windows-mirror-path]
Needs Python 3 and util-linux nsenter when Steam uses an isolated runtime.
"""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

# Pass runtime settings, never the game's entire environment or account tokens.
RUNTIME_ENV = (
    "WINEPREFIX", "WINEFSYNC", "WINEESYNC", "WINEDLLPATH", "WINELOADER",
    "LD_LIBRARY_PATH", "DISPLAY", "XAUTHORITY", "XDG_RUNTIME_DIR", "PATH",
)


def find_game(proc=Path("/proc")):
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            if entry.stat().st_uid != os.getuid():
                continue
            command = (entry / "cmdline").read_bytes().split(b"\0", 1)[0]
            executable = command.decode(errors="replace").replace("\\", "/").rsplit("/", 1)[-1]
            if executable.casefold() != "le mans ultimate.exe":
                continue
            environment = dict(
                item.split("=", 1) for item in (entry / "environ").read_bytes().decode().split("\0")
                if "=" in item
            )
            if environment.get("WINEPREFIX") and environment.get("WINELOADER"):
                return int(entry.name), environment
        except (OSError, UnicodeError):
            # A process can exit while /proc is being inspected.
            continue
    raise RuntimeError("Start Le Mans Ultimate through Steam before starting the bridge.")


def launch_command(pid, environment, bridge, arguments, namespaces):
    values = {key: environment[key] for key in RUNTIME_ENV if key in environment}
    values["WINEDEBUG"] = os.environ.get("WINEDEBUG", "-all")
    wine = environment["WINELOADER"]
    if namespaces:
        nsenter = shutil.which("nsenter")
        if not nsenter:
            raise RuntimeError("Steam uses an isolated runtime; install util-linux (nsenter).")
        # Set the game's library paths only AFTER nsenter has entered the
        # runtime, otherwise they can break the host's nsenter executable.
        command = [nsenter, "--target", str(pid), *namespaces, "--preserve-credentials",
                   "--root", "--wd", "--", "/usr/bin/env"]
        command += [f"{key}={value}" for key, value in values.items()]
        command += [wine, str(bridge), *arguments]
        return command, os.environ.copy()
    return [wine, str(bridge), *arguments], {**os.environ, **values, "LD_PRELOAD": ""}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bridge", type=Path)
    parser.add_argument("mirror_path", nargs="?")
    args = parser.parse_args()
    bridge = args.bridge.expanduser().resolve()
    if not bridge.is_file():
        parser.error(f"bridge executable does not exist: {bridge}")
    try:
        pid, environment = find_game()
        namespaces = [flag for kind, flag in [("user", "--user"), ("mnt", "--mount")]
                      if os.readlink(f"/proc/{pid}/ns/{kind}") != os.readlink(f"/proc/self/ns/{kind}")]
        arguments = [args.mirror_path] if args.mirror_path else []
        command, child_env = launch_command(pid, environment, bridge, arguments, namespaces)
        print(f"Attaching bridge to LMU process {pid}", flush=True)
        # Some Proton builds crash while resolving inherited regular-file stdio
        # outside the container. Give Wine a pipe even if this helper is logged
        # to a host file, then forward the output ourselves.
        child = subprocess.Popen(command, env=child_env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        try:
            while chunk := child.stdout.read1(4096):
                sys.stdout.buffer.write(chunk)
                sys.stdout.buffer.flush()
            return child.wait()
        except KeyboardInterrupt:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
            return 130
    except (OSError, RuntimeError) as error:
        print(f"lmu-bridge: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
