# Manga Vault

A manga and web novel aggregator that works everywhere, desktop, mobile, and browser. Run it locally with no server, or self-host for syncing across devices.

---

## What it does

- **Read anything.** Manga in a paged reader, novels with adjustable typography, RTL support, and per-work style overrides. Download chapters for offline reading.
- **Scrape anything.** WASM or Lua plugins from community repositories — zero sources bundled, install from settings.
- **Track everything.** Sync reading progress to AniList, MyAnimeList, or Kitsu.
- **Translate novels.** Local inference via Ollama, or bring your own API key. Community glossary refines translations over time.
- **Migrate sources.** Moving from a dead scraper? Match works by title and carry read state to a new source.

---

## Desktop and mobile apps

Download from [Releases](https://github.com/Dewn5228/manga-vault/releases) — Linux, macOS, Windows, Android, iOS.

In **local mode** (default), the app runs entirely on your device. SQLite on disk, no login, no server process, no network calls unless you import a work. Install plugins, build your library, read offline, it all works without an internet connection.

**Remote mode** connects to a hosted Manga Vault server for syncing your library and progress across devices. Link your devices through Settings, and the sync happens in the background.

**Local archive reading** supports CBZ files and image folders, add a folder path in Settings → Local, and browse your existing collection.

### Reader features

- **Paged mode** for manga (single page per screen), continuous scroll for novels
- **Per-work overrides**: font size, line spacing, image margins, image gap, each work remembers its own settings
- **RTL support** for right-to-left manga
- **Resume**: the reader remembers your scroll position per chapter
- **Keyboard nav**: arrow keys switch chapters, Escape returns to work page
- **Notifications**: opt-in alerts when library works get new chapters (configurable in Settings)

---

## Server and syncing

If you want your library and progress available on multiple devices, run the server and switch to remote mode on each client.

### Docker (recommended)

```bash
cd deploy
cp .env.example .env      # fill in SECRET_KEY, POSTGRES_PASSWORD
docker compose up -d
```

Requires: Docker, PostgreSQL. The web frontend is a separate static deployment (Cloudflare Pages or any static host).

### Single binary

```bash
cargo build --release
./target/release/manga-vault          # SQLite by default
DATABASE_URL=postgres://... ./target/release/manga-vault
```

### Web UI

The web frontend is a separate static site, pointed at the server via `PUBLIC_API_URL`:

```bash
cd clients/web
pnpm install && pnpm build:cloudflare   # Cloudflare Pages (recommended)
# or
pnpm install && pnpm build:static       # any static host (nginx, Caddy, etc.)
```

---

## Plugins

Manga Vault ships no scrapers. Add a repository URL in Settings → Sources:

```
https://github.com/Dewn5228/manga-vault-plugins/releases/download/latest/repo.json
```

The community repository is auto-published by GitHub Actions on every push. Plugins are self-contained WASM components or Lua scripts, no inter-plugin dependencies, sandboxed at runtime.

### Writing a plugin

Implement the Source trait via WIT (WASM) or Lua. Host standard library available to all plugins:

```
http.get(url)           html.find(doc, selector)
flare_solverr.get(url)  html.attr(elem, name)
```

No shared utils at runtime, shared code stays in the author's build tooling.

---

## Build from source

```bash
# Server
git clone https://github.com/Dewn5228/manga-vault.git && cd manga-vault
cargo build --release

# Web UI
cd clients/web && pnpm install && pnpm build:static

# Desktop / mobile
cd clients/flutter && flutter build linux   # or macos, windows, apk, ios

# WASM plugins
rustup target add wasm32-wasip2
cargo component build --target wasm32-wasip2 --release
```

---

## CI/CD

- **CI** (`ci.yml`): Rust clippy + tests, web typecheck + lint, Flutter analyze
- **Desktop** (`desktop.yml`): Linux/macOS/Windows builds on tag push → release archives
- **Mobile** (`mobile.yml`): Android APK + unsigned iOS on tag push
- **Plugins** (`plugin-repo/.github/workflows/publish.yml`): WASM builds → `.mvplug` artifacts → GitHub Release + `repo.json` commit

---

## Environment variables (server)

| Variable | Default | Purpose |
|----------|---------|---------|
| `DATABASE_URL` | `sqlite://manga-vault.db` | SQLite or Postgres connection |
| `BIND_ADDR` | `127.0.0.1:18080` | API listen address |
| `SECRET_KEY` | — | Encrypts tracker/translation credentials |
| `ADMIN_USERNAME` | — | Grants plugin management access |
| `OLLAMA_ENDPOINT` | — | Instance-wide translation via Ollama |
| `TRANSLATION_ENABLED` | `true` | Kill switch for translation |
| `PLUGINS_DIR` | `./plugins` | Plugin storage path |
| `IMAGE_CACHE_MB` | `512` | Proxy cache size for images |
| `FLARESOLVERR_URL` | — | Byparr / FlareSolverr for Cloudflare sources |
| `CORS_ORIGINS` | — | Allowed cross-origin origins |
| `PUBLIC_API_URL` | `` | API URL for the web UI |

---

## License

[GPL-3.0](LICENSE.md)
