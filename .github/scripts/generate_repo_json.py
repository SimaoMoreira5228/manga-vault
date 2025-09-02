#!/usr/bin/env python3
import os
import sys
import re
import json
from pathlib import Path
from typing import Optional, Tuple

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

API_BASE = "https://api.github.com"
HEADERS = {
    "Authorization": f"token {GITHUB_TOKEN}",
    "Accept": "application/vnd.github+json",
}

ROOT = Path(os.getenv("GITHUB_WORKSPACE", os.getcwd()))

RUST_SCRAPERS = ["mangaread_org", "manga_dex", "hari_manga"]
LUA_SCRAPERS = ["manhuafast", "natomanga", "mangabuddy", "mangakakalot"]

ALL = [{"id": s, "type": "scraper-rust-wasm"} for s in RUST_SCRAPERS] + [
    {"id": s, "type": "lua-plugin"} for s in LUA_SCRAPERS
]


def semver_tuple(v: str) -> Tuple[int, int, int]:
    m = re.match(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?", v)
    if not m:
        return (0, 0, 0)
    return (int(m.group(1) or 0), int(m.group(2) or 0), int(m.group(3) or 0))


def classify_state(version: str) -> str:
    try:
        major, minor, patch = semver_tuple(version)
        if major > 0:
            return "stable"
        if minor > 0:
            return "beta"
        return "alpha"
    except Exception:
        return "alpha"


def list_releases():
    url = f"{API_BASE}/repos/{OWNER}/{REPO_NAME}/releases?per_page=200"
    r = requests.get(url, headers=HEADERS)
    r.raise_for_status()
    return r.json()


def find_best_release_for_prefix(releases_json, prefix: str) -> Optional[dict]:
    best = None
    best_ver = None

    for r in releases_json:
        tag = (r.get("tag_name") or "").strip()
        if not tag.startswith(f"{prefix}@v"):
            continue
        version = tag.split("@v")[1]
        if best is None or semver_tuple(version) > semver_tuple(best_ver or "0.0.0"):
            best = r
            best_ver = version
    return best


def pick_asset_url(release_json: dict, wanted_ext: str) -> Optional[Tuple[str, str]]:
    assets = release_json.get("assets", [])

    for a in assets:
        name = a.get("name", "")
        if name.lower().endswith(wanted_ext):
            return name, a.get("browser_download_url")

    for a in assets:
        name = a.get("name", "")
        if wanted_ext in name.lower():
            return name, a.get("browser_download_url")
    return None


releases = list_releases()
repo_content = {"name": "dewn_plugins", "plugins": []}

for item in ALL:
    pid = item["id"]
    ptype = item["type"]
    rel = find_best_release_for_prefix(releases, pid)
    if not rel:
        print(f"Warning: no release found for {pid}; skipping")
        continue

    tag = rel.get("tag_name", "")
    version = tag.split("@v")[1] if "@v" in tag else "unknown"

    if ptype == "scraper-rust-wasm":
        picked = pick_asset_url(rel, ".wasm")
        if not picked:
            print(f"Warning: no .wasm asset found for release {tag} ({pid}); skipping")
            continue
        asset_name, url = picked
        repo_content["plugins"].append(
            {
                "name": pid,
                "urls": {"wasm": url},
                "version": version,
                "state": "updated",
                "build_state": classify_state(version),
            }
        )
        print(f"Added {pid} -> {asset_name}")
    elif ptype == "lua-plugin":
        picked = pick_asset_url(rel, ".lua")
        if not picked:
            print(f"Warning: no .lua asset found for release {tag} ({pid}); skipping")
            continue
        asset_name, url = picked
        repo_content["plugins"].append(
            {
                "name": pid,
                "urls": {"lua": url},
                "version": version,
                "state": "updated",
                "build_state": classify_state(version),
            }
        )
        print(f"Added {pid} -> {asset_name}")
    else:
        print("Unknown type", ptype)

out_path = ROOT / "repo.json"
tmp = json.dumps(repo_content, indent=2, ensure_ascii=False) + "\n"
if out_path.exists():
    old = out_path.read_text(encoding="utf-8")
    if old == tmp:
        print("repo.json unchanged — nothing to commit.")
        exit(0)

out_path.write_text(tmp, encoding="utf-8")
print("Done.")
