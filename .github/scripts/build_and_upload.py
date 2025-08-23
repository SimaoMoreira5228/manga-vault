#!/usr/bin/env python3
import os, sys, json, argparse, subprocess, shutil
from pathlib import Path

try:
    import requests
except Exception:
    print("Install requests (pip).")
    sys.exit(1)

GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
if not GITHUB_TOKEN:
    print("GITHUB_TOKEN missing.")
    sys.exit(1)

REPO = os.getenv("GITHUB_REPOSITORY")
if not REPO:
    print("GITHUB_REPOSITORY missing.")
    sys.exit(1)
OWNER, REPO_NAME = REPO.split("/")

MANIFEST_FILE = Path("release_manifest.json")


def run(cmd, cwd=None):
    print("> " + " ".join(cmd))
    subprocess.run(cmd, cwd=cwd, check=True)


def upload_asset(upload_url_template: str, asset_path: Path):
    url = upload_url_template.split("{")[0]
    params = {"name": asset_path.name}
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Content-Type": "application/octet-stream",
    }
    with open(asset_path, "rb") as f:
        r = requests.post(url, headers=headers, params=params, data=f)
    r.raise_for_status()
    return r.json()


def find_artifact(bin_name: str):
    base = Path("target") / "release"
    candidate = base / bin_name
    if candidate.exists():
        return candidate

    if (candidate.with_suffix(".exe")).exists():
        return candidate.with_suffix(".exe")

    alt = base / bin_name
    return alt if alt.exists() else None


def platform_suffix(platform):
    if platform == "linux":
        return "linux-x86_64"
    if platform == "windows":
        return "windows-x86_64"
    if platform == "macos-x86":
        return "macos-x86_64"
    if platform == "macos-arm":
        return "macos-aarch64"
    return platform


def build_binary(crate, bin_name, pkg_path: str, platform):
    print(f"Building {crate} on {platform} (native build)")
    run(["cargo", "build", "-p", crate, "--release"], cwd=pkg_path)
    artifact = Path("target") / "release" / bin_name

    if platform == "windows" and not artifact.exists():
        alt = artifact.with_suffix(".exe")
        if alt.exists():
            artifact = alt
    if not artifact.exists():
        alt2 = Path("target") / "release" / crate
        if alt2.exists():
            artifact = alt2
    if not artifact.exists():
        raise FileNotFoundError(f"artifact for {crate} not found in target/release")
    return artifact.resolve()


def zip_website(build_dir: Path, out_zip: Path):
    if not build_dir.exists():
        raise FileNotFoundError("Website build directory missing")
    import zipfile

    with zipfile.ZipFile(out_zip, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for p in build_dir.rglob("*"):
            if p.is_file():
                zf.write(p, p.relative_to(build_dir))


ap = argparse.ArgumentParser()
ap.add_argument(
    "--platform",
    required=True,
    choices=["linux", "windows", "macos-x86", "macos-arm"],
)
args = ap.parse_args()
platform = args.platform

if not MANIFEST_FILE.exists():
    print("release_manifest.json not found - nothing to build")
    sys.exit(0)

manifest = json.loads(MANIFEST_FILE.read_text(encoding="utf-8"))
created = manifest.get("created_releases", [])
if not created:
    print("manifest empty - nothing to build")
    sys.exit(0)

for entry in created:
    if entry.get("type") == "cargo-bin":
        crate = entry.get("crate")
        bin_name = entry.get("bin_name") or crate
        pkg_path = entry.get("path") or "."
        try:
            artifact = build_binary(crate, bin_name, pkg_path, platform)
            suffix = platform_suffix(platform)
            out_name = f"{bin_name}-{suffix}{'.exe' if artifact.suffix=='.exe' else ''}"
            out_path = Path(out_name)
            shutil.copy2(artifact, out_path)
            print("Uploading", out_path)
            upload_asset(entry["upload_url"], out_path)
            out_path.unlink(missing_ok=True)
            print("Uploaded", out_name)
        except Exception as e:
            print("Failed to build/upload", crate, ":", e)
    elif entry.get("type") == "bun-app" and platform == "linux":
        website_dir = Path(entry.get("path"))
        run(["bun", "install"], cwd=website_dir)

        website_build_dir = website_dir / "build"
        print("Building bun app in", website_dir)
        run(["bun", "run", "build"], cwd=website_dir)
        if website_build_dir.exists():
            try:
                zip_website(website_build_dir, entry.get("zip_name"))
                upload_asset(entry["upload_url"], Path(entry.get("zip_name")))
                Path(entry.get("zip_name")).unlink(missing_ok=True)
                print("Uploaded", entry.get("zip_name"))
            except Exception as e:
                print("Failed to zip/upload bun app:", e)
