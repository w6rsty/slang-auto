"""Install the pinned, checksum-verified upstream SDK for CI (Python 3.12+)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import tarfile
import tempfile
import urllib.request
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    config = json.loads(Path(__file__).with_name("slang-sdk.json").read_text())
    system = platform.system()
    if platform.machine().lower() not in ("amd64", "x86_64"):
        raise SystemExit("CI SDK assets support x86_64 only")
    asset = config["assets"][system]
    url = (
        "https://github.com/shader-slang/slang/releases/download/"
        f"v{config['version']}/{asset['file']}"
    )
    destination = args.destination.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    if any(destination.iterdir()):
        raise SystemExit(f"SDK destination must be empty: {destination}")
    with tempfile.TemporaryDirectory(prefix="slang-download-") as temp:
        archive = Path(temp) / asset["file"]
        request = urllib.request.Request(url, headers={"User-Agent": "slang-auto-ci"})
        with urllib.request.urlopen(request, timeout=120) as response, archive.open("wb") as output:
            while chunk := response.read(1024 * 1024):
                output.write(chunk)
        actual = hashlib.sha256(archive.read_bytes()).hexdigest()
        if actual != asset["sha256"]:
            raise SystemExit(f"SDK checksum mismatch: {actual}")
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as compressed:
                for member in compressed.namelist():
                    if not (destination / member).resolve().is_relative_to(destination):
                        raise SystemExit(f"Invalid archive member: {member}")
                compressed.extractall(destination)
        else:
            with tarfile.open(archive) as compressed:
                compressed.extractall(destination, filter="data")
    if not (destination / "include" / "slang.h").is_file():
        raise SystemExit("Unexpected SDK archive layout: include/slang.h is missing")
    print(f"Verified Slang {config['version']}: {destination}")
    if "GITHUB_ENV" in os.environ:
        with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as env:
            env.write(f"SLANG_DIR={destination}\n")
            if system == "Linux":
                env.write(f"LD_LIBRARY_PATH={destination / 'lib'}:{destination / 'bin'}\n")
        with open(os.environ["GITHUB_PATH"], "a", encoding="utf-8") as path:
            path.write(f"{destination / 'bin'}\n")


if __name__ == "__main__":
    main()
