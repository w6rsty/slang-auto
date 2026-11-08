"""Check that an unchanged build neither recompiles nor reruns build scripts."""

import json
import subprocess
from pathlib import Path


def build():
    result = subprocess.run(
        ["cargo", "build", "--locked", "-p", "slang-auto", "--example", "compile",
         "--message-format=json"],
        text=True,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        check=True,
    )
    artifacts = []
    build_scripts = {}
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact":
            artifacts.append(message)
        elif message.get("reason") == "build-script-executed":
            # Cargo replays this event even for fresh scripts. The output file's
            # mtime changes only when Cargo actually executes the script again.
            output = Path(message["out_dir"]).parent / "output"
            build_scripts[str(output)] = output.stat().st_mtime_ns
    return artifacts, build_scripts


def main():
    _, before = build()
    artifacts, after = build()
    stale = [item["target"]["name"] for item in artifacts if not item["fresh"]]
    rerun = [path for path, mtime in after.items() if before.get(path) != mtime]
    if not artifacts or stale or rerun:
        raise SystemExit(
            f"Unchanged build was not fresh: recompiled={stale}, rerun={rerun}"
        )
    print(f"Verified unchanged build: {len(artifacts)} fresh artifacts, no build scripts rerun")


if __name__ == "__main__":
    main()
