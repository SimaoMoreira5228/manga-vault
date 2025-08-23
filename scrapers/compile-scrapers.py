import json
import subprocess
import os
import sys
import requests
from typing import Optional, Tuple


rust_scrapers = ["mangaread_org", "manga_dex", "hari_manga"]
lua_scrapers = ["manhuafast", "natomanga"]

current_path = os.path.dirname(os.path.abspath(__file__))
manga_vault_path = os.path.dirname(current_path)


def build_rust(scraper_name: str) -> Optional[str]:
    try:
        print(f"Building {scraper_name}...")

        output_file = f"{scraper_name}.wasm"

        subprocess.run(
            [
                "cargo",
                "component",
                "build",
                "--target",
                "wasm32-wasip1",
                "--release",
                "--package",
                scraper_name,
            ],
            cwd=manga_vault_path,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

        built_path = os.path.join(
            manga_vault_path, "target", "wasm32-wasip1", "release", output_file
        )
        return built_path if os.path.exists(built_path) else None

    except (FileNotFoundError, subprocess.CalledProcessError):
        return None


def upload_to_catbox(file_path: str) -> Optional[str]:
    print(f"Uploading {os.path.basename(file_path)}...")
    try:
        with open(file_path, "rb") as f:
            files = {"fileToUpload": (os.path.basename(file_path), f)}
            data = {"reqtype": "fileupload"}
            resp = requests.post(
                "https://catbox.moe/user/api.php", files=files, data=data, timeout=60
            )
        resp.raise_for_status()
        text = resp.text.strip()
        return text if text.startswith("https://") else None
    except (requests.RequestException, FileNotFoundError) as e:
        return None


def get_plugin_info(
    path: str, is_lua: bool = False
) -> Tuple[Optional[str], Optional[str]]:
    try:
        if is_lua:
            with open(path, "r", encoding="utf-8") as file:
                version = None
                for line in file:
                    if "PLUGIN_VERSION" in line:
                        parts = line.split("=")
                        if len(parts) >= 2:
                            version = parts[1].strip().strip('"').strip("'")
                        break
        else:
            with open(os.path.join(path, "Cargo.toml"), "r", encoding="utf-8") as file:
                version = next(
                    (
                        line.split("=")[1].strip().strip('"').strip("'")
                        for line in file
                        if line.strip().startswith("version")
                    ),
                    None,
                )

        if not version:
            raise ValueError("version not found")

        parts = list(map(int, version.split(".")))
        if parts[0] > 0:
            state = "stable"
        elif parts[1] > 0:
            state = "beta"
        else:
            state = "alpha"

        return version, state

    except (FileNotFoundError, StopIteration, ValueError, IndexError):
        return None, None


def process_scraper(scraper: str, is_lua: bool = False) -> dict:
    try:
        if is_lua:
            path = os.path.join(current_path, scraper, f"{scraper}.lua")
            version, state = get_plugin_info(path, is_lua=True)
            if not (url := upload_to_catbox(path)):
                raise RuntimeError("Upload failed")

            return {
                "name": scraper,
                "urls": {"lua": url},
                "version": version,
                "state": "updated",
                "build_state": state,
            }

        else:
            scraper_path = os.path.join(current_path, scraper)
            version, state = get_plugin_info(scraper_path)

            if not (build_path := build_rust(scraper)):
                raise RuntimeError("build failed")
            if not (url := upload_to_catbox(build_path)):
                raise RuntimeError("upload failed")

            return {
                "name": scraper,
                "urls": {"wasm": url},
                "version": version,
                "state": "updated",
                "build_state": state,
            }

    except Exception as e:
        raise RuntimeError(f"Failed processing {scraper}: {str(e)}")


repo_content = {"name": "dewn_plugins", "plugins": []}

try:
    for scraper in rust_scrapers:
        repo_content["plugins"].append(process_scraper(scraper))

    for scraper in lua_scrapers:
        repo_content["plugins"].append(process_scraper(scraper, is_lua=True))

    with open(os.path.join(manga_vault_path, "repo.json"), "w", encoding="utf-8") as f:
        json.dump(repo_content, f, indent=2)

    print("All builds and uploads completed successfully!")
    sys.exit(0)

except Exception as e:
    print(f"ERROR: {e}", file=sys.stderr)
    print("Aborting due to failure - no repo.json created", file=sys.stderr)
    sys.exit(1)
