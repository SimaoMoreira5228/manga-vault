#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from pathlib import Path

REPO = os.getenv("GITHUB_REPOSITORY", "")
if not REPO:
  print("GITHUB_REPOSITORY missing")
  sys.exit(1)

ASSETS_DIR = Path("assets")
MANIFEST_FILE = Path("release_manifest.json")

PLATFORM_SUFFIXES = [
  "linux-x86_64",
  "windows-x86_64",
  "macos-x86_64",
  "macos-aarch64",
]


def run(cmd, **kwargs):
  print("> " + " ".join(cmd), flush=True)
  subprocess.run(cmd, check=True, **kwargs)


def expected_assets(entry):
  typ = entry["type"]
  if typ == "cargo-bin":
    bin_name = entry.get("bin_name") or entry["id"]
    result = []
    for suffix in PLATFORM_SUFFIXES:
      name = f"{bin_name}-{suffix}"
      if suffix == "windows-x86_64":
        name += ".exe"
      result.append(name)
    return result
  elif typ == "scraper-rust-wasm":
    return [f"{entry['crate']}.wasm"]
  elif typ == "lua-plugin":
    return [entry["lua_file"]]
  elif typ == "web-app":
    return [entry["zip_name"]]
  return []


def main():
  if not ASSETS_DIR.is_dir():
    print(f"assets dir {ASSETS_DIR} not found — nothing to release")
    return

  if not MANIFEST_FILE.exists():
    print("release_manifest.json not found — nothing to release")
    return

  manifest = json.loads(MANIFEST_FILE.read_text())
  entries = manifest.get("created_releases", [])
  if not entries:
    print("No releases to create")
    return

  available = {f.name for f in ASSETS_DIR.iterdir() if f.is_file()}
  print(f"Available assets ({len(available)}): {sorted(available)}")

  created = 0
  skipped = 0

  for entry in entries:
    tag = entry["tag_name"]
    expected = expected_assets(entry)
    missing = [a for a in expected if a not in available]

    if missing:
      print(f"SKIP {tag}: missing {missing}")
      skipped += 1
      continue

    print(f"CREATE {tag}: all {len(expected)} assets ready")

    try:
      exists = (
        subprocess.run(
          ["gh", "release", "view", tag, "--repo", REPO],
          capture_output=True,
          text=True,
        ).returncode
        == 0
      )

      if exists:
        print(f"  Release {tag} already exists — skipping creation, uploading missing assets")
        run(["gh", "release", "upload", tag, "--repo", REPO, "--clobber"] + [str(ASSETS_DIR / a) for a in expected])
      else:
        asset_paths = [str(ASSETS_DIR / a) for a in expected]
        run(
          [
            "gh",
            "release",
            "create",
            tag,
            "--repo",
            REPO,
            "--title",
            entry.get("name", tag),
            "--notes",
            entry.get("body", ""),
          ]
          + asset_paths
        )

      created += 1
      print(f"  OK {tag}")
    except subprocess.CalledProcessError as e:
      print(f"FAIL {tag}: {e}")
      skipped += 1

  print(f"Done: {created} releases created, {skipped} skipped")


if __name__ == "__main__":
  main()
