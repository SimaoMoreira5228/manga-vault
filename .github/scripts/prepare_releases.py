#!/usr/bin/env python3
import os, sys, re, json, subprocess
from pathlib import Path
from typing import Optional, Tuple

try:
    import tomllib
except Exception:
    import tomli as tomllib

try:
    import requests
except Exception:
    print("Install requests (pip).")
    sys.exit(1)

GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
if not GITHUB_TOKEN:
    print("GITHUB_TOKEN missing. Exiting.")
    sys.exit(1)

REPO = os.getenv("GITHUB_REPOSITORY")
if not REPO:
    print("GITHUB_REPOSITORY missing. Exiting.")
    sys.exit(1)
OWNER, REPO_NAME = REPO.split("/")

WORKSPACE = os.getenv("GITHUB_WORKSPACE", os.getcwd())
os.chdir(WORKSPACE)

PACKAGES = [
    {
        "id": "gql-api",
        "path": "apps/http/gql",
        "type": "cargo-bin",
        "crate": "gql-api",
        "bin_name": "manga-vault-gql",
        "tag_prefix": "gql-api",
    },
    {
        "id": "scheduler",
        "path": "apps/scheduler",
        "type": "cargo-bin",
        "crate": "scheduler",
        "bin_name": "manga-vault-scheduler",
        "tag_prefix": "scheduler",
    },
    {
        "id": "website-server",
        "path": "apps/website/server",
        "type": "cargo-bin",
        "crate": "website-server",
        "bin_name": "manga-vault-website-server",
        "tag_prefix": "website-server",
    },
    {
        "id": "hari_manga",
        "path": "scrapers/hari_manga",
        "type": "scraper-rust-wasm",
        "crate": "hari_manga",
        "tag_prefix": "hari_manga",
    },
    {
        "id": "manga_dex",
        "path": "scrapers/manga_dex",
        "type": "scraper-rust-wasm",
        "crate": "manga_dex",
        "tag_prefix": "manga_dex",
    },
    {
        "id": "mangaread_org",
        "path": "scrapers/mangaread_org",
        "type": "scraper-rust-wasm",
        "crate": "mangaread_org",
        "tag_prefix": "mangaread_org",
    },
    {
        "id": "manhuafast",
        "path": "scrapers/manhuafast",
        "type": "lua-plugin",
        "lua_file": "manhuafast.lua",
        "tag_prefix": "manhuafast",
    },
    {
        "id": "natomanga",
        "path": "scrapers/natomanga",
        "type": "lua-plugin",
        "lua_file": "natomanga.lua",
        "tag_prefix": "natomanga",
    },
]

MAIN_PACKAGE = {
    "crate": "manga-vault",
    "bin_name": "manga-vault",
    "path": ".",
    "tag_prefix": "manga-vault",
}

API_BASE = "https://api.github.com"


def parse_version_from_cargo_toml(cargo_path: Path) -> Optional[str]:
    try:
        with cargo_path.open("rb") as f:
            data = tomllib.load(f)
        pkg = data.get("package")
        if pkg and "version" in pkg:
            return pkg["version"]
        if "version" in data:
            return data["version"]
    except Exception:
        pass
    return None


def parse_version_from_lua(lua_path: Path) -> Optional[str]:
    try:
        txt = lua_path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return None
    m = re.search(r'PLUGIN_VERSION\s*=\s*["\']([\d\.]+[^\n"\']*)["\']', txt)
    return m.group(1).strip() if m else None


def semver_tuple(v: str) -> Tuple[int, int, int]:
    m = re.match(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?", v)
    if not m:
        return (0, 0, 0)
    return (int(m.group(1) or 0), int(m.group(2) or 0), int(m.group(3) or 0))


def is_newer(local: str, remote: Optional[str]) -> bool:
    if not remote:
        return True
    return semver_tuple(local) > semver_tuple(remote)


def github_list_releases():
    url = f"{API_BASE}/repos/{OWNER}/{REPO_NAME}/releases?per_page=200"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github+json",
    }
    r = requests.get(url, headers=headers)
    r.raise_for_status()
    return r.json()


def find_latest_release_version_for_prefix(releases_json, prefix: str) -> Optional[str]:
    best = None
    for r in releases_json:
        tag = r.get("tag_name", "")
        m = re.search(rf"{re.escape(prefix)}@v?([\d]+\.[\d]+\.[\d]+)", tag)
        if not m:
            m = re.search(r"v?([\d]+\.[\d]+\.[\d]+)", tag)
        if m:
            ver = m.group(1)
            if best is None or semver_tuple(ver) > semver_tuple(best):
                best = ver
    return best


def run(cmd, cwd=None):
    print("> " + " ".join(cmd))
    subprocess = __import__("subprocess")
    subprocess.run(cmd, cwd=cwd, check=True)


def tag_and_push(tag_name: str):
    run(["git", "tag", "-a", tag_name, "-m", f"Release {tag_name}"])
    run(["git", "push", "origin", tag_name])


def create_github_release(tag_name: str, name: str, body: str):
    url = f"{API_BASE}/repos/{OWNER}/{REPO_NAME}/releases"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github+json",
    }
    payload = {"tag_name": tag_name, "name": name, "body": body, "prerelease": False}
    r = requests.post(url, headers=headers, json=payload)
    r.raise_for_status()
    return r.json()


def read_local_version_for_pkg(entry):
    p = Path(entry.get("path", "."))

    if entry["type"] in ("cargo-bin", "scraper-rust-wasm"):
        cargo = p / "Cargo.toml"
        return parse_version_from_cargo_toml(cargo)
    elif entry["type"] == "lua-plugin":
        lua = p / entry["lua_file"]
        return parse_version_from_lua(lua)
    return None


print("Preparing releases...")
releases_json = github_list_releases()
manifest = {"created_releases": []}

for entry in PACKAGES:
    local_ver = read_local_version_for_pkg(entry)
    if not local_ver:
        print("No local version for", entry["id"], "; skipping")
        continue
    remote_ver = find_latest_release_version_for_prefix(
        releases_json, entry["tag_prefix"]
    )
    print(entry["id"], "local", local_ver, "remote", remote_ver)
    if is_newer(local_ver, remote_ver):
        tag = f"{entry['tag_prefix']}@v{local_ver}"
        try:
            tag_and_push(tag)
        except Exception as e:
            print("Warning: tag push error:", e)
        try:
            rel = create_github_release(
                tag,
                f"{entry['id']} v{local_ver}",
                f"Automatic release for {entry['id']} v{local_ver}",
            )
            manifest["created_releases"].append(
                {
                    "id": entry["id"],
                    "type": entry["type"],
                    "tag_name": tag,
                    "version": local_ver,
                    "upload_url": rel["upload_url"],
                    "crate": entry.get("crate"),
                    "bin_name": entry.get("bin_name"),
                    "path": entry.get("path"),
                    "lua_file": entry.get("lua_file"),
                }
            )
            print("Created release for", entry["id"], tag)
        except Exception as e:
            print("Failed to create release for", entry["id"], e)

if manifest["created_releases"]:
    cargo = Path("Cargo.toml")
    if cargo.exists():
        text = cargo.read_text(encoding="utf-8")
        m = re.search(r'(?m)^version\s*=\s*["\']([\d]+\.[\d]+\.[\d]+)["\']', text)
        if m:
            old = m.group(1)
            major, minor, patch = semver_tuple(old)
            new = f"{major}.{minor}.{patch+1}"
            new_text = re.sub(
                r'(?m)^(version\s*=\s*)["\'][\d]+\.[\d]+\.[\d]+["\']',
                rf'\1"{new}"',
                text,
            )
            cargo.write_text(new_text, encoding="utf-8")
            tag = f"{MAIN_PACKAGE['tag_prefix']}@v{new}"
            try:
                run(["git", "tag", "-a", tag, "-m", f"Release {tag}"])
                run(["git", "push", "origin", tag])
            except Exception as e:
                print("Warning: tagging main failed:", e)
            try:
                rel = create_github_release(
                    tag,
                    f"{MAIN_PACKAGE['crate']} v{new}",
                    f"Automatic main release {new}",
                )
                manifest["created_releases"].append(
                    {
                        "id": MAIN_PACKAGE["crate"],
                        "type": "cargo-bin",
                        "tag_name": tag,
                        "version": new,
                        "upload_url": rel["upload_url"],
                        "crate": MAIN_PACKAGE["crate"],
                        "bin_name": MAIN_PACKAGE["bin_name"],
                        "path": MAIN_PACKAGE["path"],
                    }
                )
                print("Created main release", tag)
            except Exception as e:
                print("Failed to create main release:", e)
        else:
            print("Could not parse main Cargo.toml version; skipping bump")
else:
    print("No packages to release; manifest will be empty (no work for builders)")

with open("release_manifest.json", "w", encoding="utf-8") as f:
    json.dump(manifest, f, indent=2)
print("Wrote release_manifest.json with", len(manifest["created_releases"]), "entries")
