#!/usr/bin/env python3
import json
import os
import shutil
import subprocess
import sys
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
WORKSPACE = os.getenv("GITHUB_WORKSPACE", os.getcwd())
os.chdir(WORKSPACE)
API_BASE = "https://api.github.com"


def run(cmd, cwd=None):
    print("> " + " ".join(cmd))
    subprocess.run(cmd, cwd=cwd, check=True)


def ensure_git_identity():
    name = subprocess.run(
        ["git", "config", "--get", "user.name"],
        capture_output=True,
        text=True,
    )
    email = subprocess.run(
        ["git", "config", "--get", "user.email"],
        capture_output=True,
        text=True,
    )
    if not name.stdout.strip():
        run(["git", "config", "user.name", "github-actions[bot]"])
    if not email.stdout.strip():
        run([
            "git",
            "config",
            "user.email",
            "github-actions[bot]@users.noreply.github.com",
        ])


def gh_release_view(tag_name: str):
    if shutil.which("gh") is None:
        return None
    try:
        result = subprocess.run(
            [
                "gh",
                "release",
                "view",
                tag_name,
                "--repo",
                f"{OWNER}/{REPO_NAME}",
                "--json",
                "uploadUrl,tagName,name,body",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        data = json.loads(result.stdout)
        data["upload_url"] = data.get("uploadUrl")
        return data
    except subprocess.CalledProcessError:
        return None


def gh_release_create(tag_name: str, name: str, body: str):
    run([
        "gh",
        "release",
        "create",
        tag_name,
        "--repo",
        f"{OWNER}/{REPO_NAME}",
        "--title",
        name,
        "--notes",
        body,
    ])
    return gh_release_view(tag_name)


def gh_upload_asset(tag_name: str, asset_path: Path):
    run([
        "gh",
        "release",
        "upload",
        tag_name,
        str(asset_path),
        "--repo",
        f"{OWNER}/{REPO_NAME}",
        "--clobber",
    ])
    return True


def github_get_release_by_tag(tag_name: str):
    url = f"{API_BASE}/repos/{OWNER}/{REPO_NAME}/releases/tags/{tag_name}"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github+json",
    }
    r = requests.get(url, headers=headers)
    if r.status_code == 404:
        return None
    r.raise_for_status()
    return r.json()


def create_github_release(tag_name: str, name: str, body: str):
    url = f"{API_BASE}/repos/{OWNER}/{REPO_NAME}/releases"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github+json",
    }
    payload = {
        "tag_name": tag_name,
        "name": name,
        "body": body,
        "prerelease": False,
    }
    r = requests.post(url, headers=headers, json=payload)
    try:
        r.raise_for_status()
    except requests.HTTPError as exc:
        if r.status_code == 422:
            existing = github_get_release_by_tag(tag_name)
            if existing:
                return existing
        raise
    return r.json()


def tag_and_push(tag_name: str):
    ensure_git_identity()
    if subprocess.run(["git", "rev-parse", tag_name], check=False, capture_output=True).returncode == 0:
        print(f"Tag {tag_name} already exists locally.")
    else:
        run(["git", "tag", "-a", tag_name, "-m", f"Release {tag_name}"])
    try:
        run(["git", "push", "origin", tag_name])
    except subprocess.CalledProcessError as exc:
        print("Warning: tag push error:", exc)


def ensure_release(tag_name: str, name: str, body: str):
    existing = gh_release_view(tag_name)
    if existing:
        return existing

    if shutil.which("gh") is not None:
        try:
            return gh_release_create(tag_name, name, body)
        except Exception as exc:
            print("Warning: gh release creation failed:", exc)

    existing = github_get_release_by_tag(tag_name)
    if existing:
        return existing

    try:
        tag_and_push(tag_name)
    except Exception as exc:
        print("Warning: tag creation/push error:", exc)

    try:
        return create_github_release(tag_name, name, body)
    except Exception as exc:
        print("Warning: create release failed:", exc)
        existing = github_get_release_by_tag(tag_name)
        if existing:
            return existing
        raise


def upload_asset_to_release(tag_name: str, asset_path: Path, release=None):
    if shutil.which("gh") is not None:
        return gh_upload_asset(tag_name, asset_path)

    if release is None:
        release = github_get_release_by_tag(tag_name)
    upload_url = release.get("upload_url") or release.get("uploadUrl")
    if not upload_url:
        raise RuntimeError(f"Missing upload URL for release {tag_name}")
    return upload_asset(upload_url, asset_path)


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
