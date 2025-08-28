# Manga Vault
Manga Vault is a **single Rust binary** that launches a server, a scheduler for regular plugin updates, and a website to read manga/webtoons. Optional separate binaries (scheduler, API-only, website-only) are provided in Releases for people who want to run components independently.

---

## Features

* Single-binary server that includes:

  * HTTP website for reading manga/webtoons
  * GraphQL API
  * Scheduler that periodically updates plugins/repositories
* Plugin system: scrapers are distributed as separate files (`.wasm` or `.lua`) and fetched from remote repository JSON indexes.
* Repository filtering via per-repository `whitelist` / `blacklist` (configured in your local config).
* Uses WASM components with `wit-bindgen`/wasmtime and Lua plugins via `mlua` for easy cross-language plugin development.

---

## Quick install

1. Download the binary you want from Releases. There are multiple release artifacts: the main `manga-vault` binary (server + scheduler + website) and separate binaries for the scheduler/API/website if you prefer.
2. Put the binary in a folder you control and run it.
3. By default the app starts the services and listens on the following local ports (unless changed in config):

   * Website UI: `http://localhost:5227`
   * GraphQL API: `http://localhost:5228`
   * GraphQL Playground: `http://localhost:5228/playground`

---

### Database support — SQLite, MySQL, PostgreSQL
Single-binary installs can use SQLite. If you run the separate binaries (scheduler / API / website) across different Docker containers or machines, use MySQL or PostgreSQL so all services share the same DB. Configure DB in `config/database.json` (example below). Note: built-in automatic backups apply only to SQLite — if you use MySQL/Postgres, run your own backup strategy.

MySQL Example `config/database.json`:
```json
{
  "backup_interval": 2,
  "backup_retention_days": 7,
  "database_backup_folder": "/path/to/backups",
  "database_url": "mysql://localhost:3306/manga-vault?user=root"
}
```

SQLite Example `config/database.json`:
```json
{
  "backup_interval": 2,
  "backup_retention_days": 7,
  "database_backup_folder": "/path/to/backups",
  "database_url": "sqlite://./manga-vault.db"
}
```

---

## Configuration (plugins)

Plugin-related settings live in the app config under the `repositories` and `plugins_folder` keys. Example minimal config:

```json
{
  "headless": null,
  "plugins_folder": "/path/to/plugins",
  "repositories": [
    { "url": "https://raw.githubusercontent.com/SimaoMoreira5228/manga-vault/refs/heads/master/repo.json" }
  ]
}
```

* `plugins_folder`: local directory where repository subfolders and downloaded plugin files will be stored.
* `repositories`: array of repository entries (currently only `url` is required). Each `url` must point to a JSON file describing the repository (format below).
* `headless`: when set to a WebDriver URL (for example `"http://localhost:4444"`) the host exposes a `headless_client` to Lua plugins so they can control a browser for JS-heavy pages. Keep `null` to disable.

---

## Repository JSON format

A repository JSON must contain a `name` and a `plugins` array. Each plugin entry should include the plugin `name`, download `urls` (either `wasm` or `lua`), a `version`, `state` and `build_state` metadata.

```json
{
  "name": "dewn_plugins",
  "plugins": [
    {
      "name": "mangaread_org",
      "urls": { "wasm": "https://.../mangaread_org.wasm" },
      "version": "0.4.2",
      "state": "updated",
      "build_state": "beta"
    },
    {
      "name": "manhuafast",
      "urls": { "lua": "https://.../manhuafast.lua" },
      "version": "0.2.0",
      "state": "updated",
      "build_state": "beta"
    }
  ]
}
```

Notes:

* Downloader prefers a `lua` URL when present, otherwise `wasm`.
* `version` prevents re-downloading unchanged plugin files.

---

## Plugin runtime expectations (developer-facing)

### WASM plugins

* Implement a component that conforms to the host scraper types (the host uses `wit-bindgen`/wasmtime component model).
* At runtime the host instantiates the component and calls functions such as:

  * `get_info()` -> returns scraper metadata (id/version)
  * `scrape_chapter(url)` -> `Vec<String>` (page image URLs)
  * `scrape_latest(page)` -> `Vec<MangaItem>`
  * `scrape_trending(page)` -> `Vec<MangaItem>`
  * `scrape_search(query, page)` -> `Vec<MangaItem>`
  * `scrape_manga(url)` -> `MangaPage`
  * `scrape_genres_list()` -> `Vec<Genre>`
* WASM plugins have access to host-provided capabilities through these interfaces:
  - **HTTP**: `get()`, `post()`, `has_cloudflare_protection()`
  - **HTML Parsing**: `find()`, `find_one()`, `text()`, `attr()`
  - **Headless Browser** (when configured): `goto()`, `find_one()`, `find_all()`, `close()`
  - **FlareSolverr Integration**: `create_session()`, `get()`

### Lua plugins

* Each Lua plugin must expose at least these globals:

  * `PLUGIN_NAME` (string)
  * `PLUGIN_VERSION` (string)
* And these functions:

  * `Get_info()` -> returns scraper info
  * `Scrape_chapter(url)` -> returns a list of page URLs
  * `Scrape_latest(page)` -> returns a list of `MangaItem`
  * `Scrape_trending(page)` -> returns a list of `MangaItem`
  * `Scrape_search(query, page)` -> returns a list of `MangaItem`
  * `Scrape_manga(url)` -> returns a `MangaPage`
  * `Scrape_genres_list()` -> returns a list of genres
* Lua plugins receive these additional global helpers:
  - **`http`**: HTTP client with methods like `get()`, `post()`, `has_cloudflare_protection()`
  - **`headless_client`**: Headless browser control (when configured)
  - **`flaresolverr`**: Cloudflare bypass capabilities
  - **`scraping`**: HTML parsing utilities
  - **Extended `string` library**: Additional methods like `split()`, `trim()`, `replace()`
  - **Extended `table` library**: Additional methods like `reverse()`

---

### Development Environment
For Lua plugin development, we recommend using these VS Code settings (`settings.json`) for better development experience:

```json
{
  "Lua.workspace.library": [
    "/path/to/manga-vault/scrapers/scraper_core/types.lua"
  ],
  "Lua.diagnostics.globals": [
    "http",
    "headless_client",
    "flaresolverr",
    "scraping",
    "PLUGIN_NAME",
    "PLUGIN_VERSION"
  ]
}
```
The `types.lua` file provides type definitions and IntelliSense support for the host-provided APIs. Place this file in your Lua workspace library path to get autocompletion and type checking.

---

## Headless mode (for JS-heavy sites)

When `headless` in config is set to a WebDriver URL (e.g. `"http://localhost:4444"`), the host exposes a `headless_client` to Lua scripts. The client provides methods such as `get(url)`, `find(selector)`, `find_all(selector)`, `close()` and element helpers like `click()` and `text()`.

Example config snippet to enable a local WebDriver (e.g. geckodriver / chromedriver):

```json
{
  "headless": "http://localhost:4444"
}
```

---

## Build from source

```bash
# install Rust (rustup)
git clone https://github.com/SimaoMoreira5228/manga-vault.git
cd manga-vault
cargo build --release
# binaries in target/release/
```

---

## Troubleshooting

* No plugins or repository errors: verify the `repositories` URLs are reachable and `plugins_folder` is writable.
* Headless failures: ensure a WebDriver is running at the configured `headless` URL and accepts headless sessions.
* WASM plugin errors: make sure the component is built for the WASM component model the host expects and that `get_info()` returns valid metadata.
* For Lua plugin errors: Verify your development environment is properly configured with the correct `types.lua` path and global variables are recognized
* For WASM component errors: Ensure your component properly implements all required WIT interfaces and handles host-provided capabilities correctly