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

### Migrating from the old manga-vault

If you have data in the old manga-vault, there's a one-shot migration binary in `tools/migrate`. It reads from your legacy MySQL **or** SQLite database and writes into a new SQLite or Postgres target. The legacy schema is identical across those backends, so only the `--from` URL changes.

```bash
# Build the tool
cargo build -p manga-vault-migrate --release

# Run the migration (legacy MySQL)
./target/release/manga-vault-migrate \
  --from mysql://user:password@localhost:3306/old_vault \
  --to sqlite://./manga-vault.db

# ...or a legacy SQLite vault, which is also accepted
./target/release/manga-vault-migrate \
  --from sqlite:///srv/old-vault/database.db \
  --to postgres://manga_vault@localhost/manga_vault
```

What it migrates:

- **Works** — manga and novels into a unified table, deduplicated by remote URL
- **Chapters** — image and novel chapters sorted by original date, duplicate URLs collapsed
- **Library entries** — favorites with category assignments carried over
- **Reading progress** — which chapters each user has read, duplicates collapsed
- **Users and categories** — carried over with new UUIDs

After migration, the tool prints a verification table comparing row counts between the legacy and new databases. If counts don't match, the legacy database should be kept as a cold backup and investigated before retiring it.

The SQLite and MySQL paths both have integration tests. The MySQL one needs a live server, so it only runs when you point it at one — it creates and drops its own `migrate_probe` database:

```bash
docker run -d --name legacy-mysql -p 127.0.0.1:3307:3306 \
  -e MYSQL_ROOT_PASSWORD=probe mysql:8

MV_MYSQL_URL=mysql://root:probe@127.0.0.1:3307/ cargo test -p manga-vault-migrate
```

```bash
# Example output:
# entity                 legacy        new
# works                       432         432  ok
# chapters                  12847       12847  ok
# library_entries             312         312  ok
# reading_progress           8102        8102  ok
# users                         3           3  ok
```

The legacy source must be MySQL (the old manga-vault supported MySQL). The target can be either SQLite or Postgres, use Postgres if you plan to run the new server in hosted mode.

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
| `DATABASE_URL` | `sqlite://./data/manga-vault.db` | SQLite or Postgres connection |
| `BIND_ADDR` | `127.0.0.1:8080` | API listen address |
| `DATA_DIR` | `./data` | Local data directory (downloads, repos) |
| `SECRET_KEY` | — | Encrypts tracker/translation credentials at rest. Optional; when unset those features return an error. Must be 64 hex chars — the server refuses to start otherwise. Changing it makes existing credentials undecryptable |
| `SESSION_TTL_DAYS` | `7` | Idle days before a session is rejected and the cookie expires |
| `SECURE_COOKIES` | `true` unless `BIND_ADDR` is loopback | Marks the session cookie `Secure` and prefixes it `__Host-`. Leave on for any deployment reachable over the network; set `false` only for plain-HTTP local development |
| `ADMIN_USERNAME` | — | Grants admin access (plugin + registration management) |
| `REGISTRATION_MODE` | `open` | `open`, `closed`, or `invite` |
| `OLLAMA_ENDPOINT` | — | Instance-wide translation via Ollama |
| `OLLAMA_MODEL` | `qwen2.5:7b` | Model used for Ollama translation |
| `TRANSLATION_ENABLED` | `true` | Kill switch for translation |
| `PLUGINS_DIR` | `./plugins` | Plugin storage path |
| `IMAGE_CACHE_MB` | `512` | Proxy cache size for images |
| `PROXY_REQUESTS_PER_MINUTE` | `240` | Per-user image proxy request budget, refilled continuously. Exhausting it returns `429` with `Retry-After` |
| `FLARESOLVERR_URL` | — | FlareSolverr for Cloudflare sources |
| `CORS_ORIGINS` | — | Comma-separated allowed origins. Setting this marks the session cookie `SameSite=None` (and forces it `Secure`) for cross-site deployments such as Cloudflare Pages. Same-origin deployments keep `SameSite=Lax` |
| `PUBLIC_API_URL` | `` | API URL for the web UI, baked in at build time (empty = same origin) |

Every error response carries a stable machine-readable `code` alongside the human-readable `error` string, so clients branch on the code instead of matching prose:

```json
{ "error": "username `dup` is already taken", "code": "username_taken" }
```

Codes: `not_found`, `bad_credentials`, `too_many_attempts`, `registration_closed`, `invite_required`, `invalid_invite`, `username_taken`, `conflict`, `source_unavailable`, `plugin_not_found`, `bad_plugin_artifact`, `upstream_failed`, `internal`, `bad_request`, `forbidden`, `too_many_requests`.

`GET /api/library`, `GET /api/me/history`, and `GET /api/me/sessions` are paginated with `?offset=&limit=`.
`/api/library` also accepts `?kind=manga|novel`, `?category_id=<uuid>`, and `?q=<text>` (case-insensitive title search),
and `total` reflects the filtered count, so a filtered view pages independently of the full library.
`limit` defaults to 50 and is clamped to 200. Each response carries `total`, `offset`, and `limit` alongside its items
(`/api/me/sessions` wraps them in `items`).

Writes authenticated by cookie are checked against the request's `Origin`: a cross-origin `POST`/`PUT`/`DELETE`/`PATCH` is rejected with `403` unless the origin is same-host or listed in `CORS_ORIGINS`. Bearer clients (Flutter, scripts) and non-browser clients that send no `Origin` are unaffected.

---

## License

[GPL-3.0](LICENSE.md)
