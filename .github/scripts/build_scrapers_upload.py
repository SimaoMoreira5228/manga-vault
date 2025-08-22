#!/usr/bin/env python3
import os, sys, json, subprocess
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


def build_wasm(crate, pkg_path):
    run(
        [
            "cargo",
            "component",
            "build",
            "--package",
            crate,
            "--target",
            "wasm32-wasip1",
            "--release",
        ],
        cwd=pkg_path,
    )
    out = Path("target") / "wasm32-wasip1" / "release" / f"{crate}.wasm"
    if out.exists():
        return out.resolve()
    raise FileNotFoundError(f"WASM build output not found for {crate}")


if not MANIFEST_FILE.exists():
    print("release_manifest.json not found")
    sys.exit(0)

manifest = json.loads(MANIFEST_FILE.read_text(encoding="utf-8"))
entries = manifest.get("created_releases", [])
for e in entries:
    t = e.get("type")
    if t == "scraper-rust-wasm":
        crate = e.get("crate")
        pkg_path = e.get("path") or "."
        try:
            artifact = build_wasm(crate, pkg_path)
            print("Uploading", artifact)
            upload_asset(e["upload_url"], artifact)
            print("Uploaded wasm for", crate)
        except Exception as ex:
            print("Failed wasm build/upload for", crate, ex)
    elif t == "lua-plugin":
        lua_file = Path(e.get("path") or ".") / e.get("lua_file")
        if lua_file.exists():
            try:
                print("Uploading lua plugin", lua_file)
                upload_asset(e["upload_url"], lua_file)
                print("Uploaded lua", lua_file.name)
            except Exception as ex:
                print("Failed uploading lua", lua_file, ex)
        else:
            print("Lua file not found:", lua_file)
    else:
        print("Skipping entry type", t)
