import json
import subprocess
import os
import sys

rust_scrapers = ["mangaread_org", "manga_dex", "hari_manga"]
lua_scrapers = ["manhuafast"]

current_path = os.path.dirname(os.path.abspath(__file__))
manga_vault_path = os.path.dirname(current_path)


def build_rust(path, target):
    try:
        with open(os.path.join(path, "Cargo.toml"), "r") as file:
            for line in file:
                if line.strip().startswith("name"):
                    project_name = line.split("=")[1].strip().strip('"')
                    break
            else:
                return None

        print(f"Building {target} for {project_name}...")

        if "windows" in target:
            output_file = f"{project_name}.dll"
        else:
            output_file = f"lib{project_name}.so"

        subprocess.run(
            ["cargo", "build", "--release", "--target", target],
            cwd=path,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

        built_path = os.path.join(path, "target", target, "release", output_file)
        return built_path if os.path.exists(built_path) else None

    except (FileNotFoundError, subprocess.CalledProcessError):
        return None


def upload_to_catbox_with_curl(file_path):
    print(f"Uploading {os.path.basename(file_path)}...")
    try:
        result = subprocess.run(
            [
                "curl",
                "-X",
                "POST",
                "https://catbox.moe/user/api.php",
                "-F",
                f"fileToUpload=@{file_path}",
                "-F",
                "reqtype=fileupload",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip() if result.stdout.startswith("https://") else None
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


def get_plugin_info(path, is_lua=False):
    try:
        if is_lua:
            with open(path, "r") as file:
                for line in file:
                    if "PLUGIN_VERSION" in line:
                        version = line.split("=")[1].strip().strip('"')
                        break
        else:
            with open(os.path.join(path, "Cargo.toml"), "r") as file:
                version = next(
                    line.split("=")[1].strip().strip('"')
                    for line in file
                    if line.strip().startswith("version")
                )

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


def process_scraper(scraper, is_lua=False):
    try:
        if is_lua:
            path = os.path.join(current_path, scraper, f"{scraper}.lua")
            version, state = get_plugin_info(path, is_lua=True)
            if not (url := upload_to_catbox_with_curl(path)):
                raise RuntimeError("Upload failed")

            return {
                "name": scraper,
                "urls": {"lua": url},
                "version": version,
                "state": "updated",
                "build_state": state,
            }

        else:
            path = os.path.join(current_path, scraper)
            version, state = get_plugin_info(path)

            targets = {
                "windows": "x86_64-pc-windows-gnu",
                "linux": "x86_64-unknown-linux-gnu",
            }

            urls = {}
            for platform, target in targets.items():
                if not (build_path := build_rust(path, target)):
                    raise RuntimeError(f"{platform} build failed")
                if not (url := upload_to_catbox_with_curl(build_path)):
                    raise RuntimeError(f"{platform} upload failed")
                urls[platform] = url

            return {
                "name": scraper,
                "urls": urls,
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

    with open(os.path.join(manga_vault_path, "repo.json"), "w") as f:
        json.dump(repo_content, f, indent=2)

    print("All builds and uploads completed successfully!")
    sys.exit(0)

except Exception as e:
    print(f"ERROR: {e}", file=sys.stderr)
    print("Aborting due to failure - no repo.json created", file=sys.stderr)
    sys.exit(1)
