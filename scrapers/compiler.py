import json
import subprocess

scrapers = ["mangaread_org", "manga_dex"]

current_path = "/".join(__file__.split("/")[:-1])
manga_vault_path = "/".join(current_path.split("/")[:-1])


def build_windows(path):
    project_name = None
    with open(f"{path}/Cargo.toml", "r") as file:
        for line in file:
            if "name" in line:
                project_name = line.split("=")[1].strip().strip('"')
                break

    subprocess.run(["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"], cwd=path)

    return f"{path}/target/x86_64-pc-windows-gnu/release/{project_name}.dll"


def build_linux(path):
    project_name = None
    with open(f"{path}/Cargo.toml", "r") as file:
        for line in file:
            if "name" in line:
                project_name = line.split("=")[1].strip().strip('"')
                break

    subprocess.run(["cargo", "build", "--release"], cwd=path)

    return f"{path}/target/release/lib{project_name}.so"


def upload_to_catbox_with_curl(file_path):
    url = "https://catbox.moe/user/api.php"
    command = [
        "curl", "-X", "POST", url,

        "-F", f"fileToUpload=@{file_path}",
        "-F", "reqtype=fileupload"
    ]

    result = subprocess.run(command, capture_output=True, text=True)

    if result.returncode == 0:
        print("Upload successful:", result.stdout)
        return result.stdout
    else:
        print("Upload failed:", result.stderr)


def get_plugin_version(path):
    with open(f"{path}/Cargo.toml", "r") as file:
        for line in file:
            if "version" in line:
                return line.split("=")[1].strip().strip('"')


def get_build_state(path):
    version = get_plugin_version(path)
    if int(version[0]) > 0:
        return "stable"
    elif int(version[2]) > 0:
        return "beta"
    else:
        return "alpha"


repo_file_content = {
    "name": "dewn_plugins",
    "plugins": []
}

for scraper in scrapers:
    path = f"{current_path}/{scraper}"
    data = {
        "name": scraper,
        "urls": {
            "windows": upload_to_catbox_with_curl(build_windows(path)),
            "linux": upload_to_catbox_with_curl(build_linux(path))
        },
        "version": get_plugin_version(path),
        "state": "updated",
        "build_state": get_build_state(path),
    },

    repo_file_content["plugins"].append(data)

json.dump(repo_file_content, open(f"{manga_vault_path}/repo.json", "w"), indent=2)
