#!/usr/bin/env python3
import os, sys, re, json
from pathlib import Path
from typing import Optional, Tuple

SCRIPT_DIR = Path(__file__).resolve().parent

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

PACKAGES_FILE = SCRIPT_DIR.parent / "packages.json"
PACKAGES = json.loads(PACKAGES_FILE.read_text())

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

  m = re.search(
    r"function\s+Get_info\s*\([^)]*\)\s*return\s*\{(.*?)\}\s*end",
    txt,
    re.DOTALL,
  )

  if not m:
    m = re.search(
      r"Get_info\s*=\s*function\s*\([^)]*\)\s*return\s*\{(.*?)\}\s*end",
      txt,
      re.DOTALL,
    )

  if not m:
    return None

  table_body = m.group(1)

  vm = re.search(r'version\s*=\s*["\']([^"\']+)["\']', table_body)
  if vm:
    return vm.group(1).strip()

  return None


def parse_version_from_package_json(package_json_path: Path) -> Optional[str]:
  try:
    with package_json_path.open("r", encoding="utf-8") as f:
      data = json.load(f)
    return data.get("version")
  except Exception:
    return None


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
    if not tag.startswith(f"{prefix}@v"):
      continue
    version = tag.split("@v")[1]
    if best is None or semver_tuple(version) > semver_tuple(best):
      best = version
  return best


def read_local_version_for_pkg(entry):
  p = Path(entry.get("path", "."))

  if entry["type"] in ("cargo-bin", "scraper-rust-wasm"):
    cargo = p / "Cargo.toml"
    return parse_version_from_cargo_toml(cargo)
  elif entry["type"] == "lua-plugin":
    lua = p / entry["lua_file"]
    return parse_version_from_lua(lua)
  elif entry["type"] == "web-app":
    package_json = p / "package.json"
    return parse_version_from_package_json(package_json)
  return None


print("Preparing release manifest...")
releases_json = github_list_releases()
manifest = {"created_releases": []}

for entry in PACKAGES:
  local_ver = read_local_version_for_pkg(entry)
  if not local_ver:
    print("No local version for", entry["id"], "; skipping")
    continue
  remote_ver = find_latest_release_version_for_prefix(releases_json, entry["tag_prefix"])
  print(entry["id"], "local", local_ver, "remote", remote_ver)
  if is_newer(local_ver, remote_ver):
    tag = f"{entry['tag_prefix']}@v{local_ver}"
    manifest["created_releases"].append(
      {
        "id": entry["id"],
        "type": entry["type"],
        "tag_name": tag,
        "version": local_ver,
        "name": f"{entry['id']} v{local_ver}",
        "body": f"Automatic release for {entry['id']} v{local_ver}",
        "bin_name": entry.get("bin_name"),
        "crate": entry.get("crate"),
        "lua_file": entry.get("lua_file"),
        "zip_name": entry.get("zip_name"),
      }
    )
    print("Prepared release manifest entry for", entry["id"], tag)

with open("release_manifest.json", "w", encoding="utf-8") as f:
  json.dump(manifest, f, indent=2)
print("Wrote release_manifest.json with", len(manifest["created_releases"]), "entries")
