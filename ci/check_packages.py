"""Audit actual .crate archives after cargo package --workspace."""
from pathlib import Path
import json
import subprocess
import tarfile
import tomllib


def main():
    root = Path(__file__).resolve().parents[1]
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=root, text=True
    ))
    license_text = (root / "LICENSE").read_bytes()
    for package in metadata["packages"]:
        name, version = package["name"], package["version"]
        prefix = f"{name}-{version}"
        archive = Path(metadata["target_directory"]) / "package" / f"{prefix}.crate"
        with tarfile.open(archive) as contents:
            files = {member.name.removeprefix(prefix + "/") for member in contents.getmembers()}
            required = {"Cargo.toml", "README.md", "LICENSE", "src/lib.rs"}
            if name == "slang-auto-sys":
                required |= {"build.rs", "generate.rs", "tests/generator.rs",
                             "docs/windows-x86_64/bindings.rs", "docs/windows-x86_64/interfaces.rs",
                             "docs/windows-x86_64/api-manifest.txt", "docs/SLANG-LICENSE",
                             "docs/SLANG-APACHE-2.0.txt"}
            else:
                required |= {"examples/compile.rs", "tests/compiler.rs"}
            assert required <= files, (name, "missing files", required - files)
            assert not any(Path(f).suffix in {".dll", ".lib", ".exe", ".obj", ".so"} for f in files)
            assert contents.extractfile(f"{prefix}/LICENSE").read() == license_text
            manifest = tomllib.loads(contents.extractfile(f"{prefix}/Cargo.toml").read().decode())
            assert manifest["package"]["license"] == "MIT"
            assert manifest["package"]["edition"] == "2024"
            assert manifest["package"]["rust-version"] == "1.85"
            assert manifest["package"]["repository"] == "https://github.com/w6rsty/slang-auto"
            if name == "slang-auto":
                dependency = manifest["dependencies"]["slang-auto-sys"]
                assert "path" not in dependency and dependency["version"] == version
        print(f"Verified {archive.name}: {len(files)} files")


if __name__ == "__main__":
    main()
