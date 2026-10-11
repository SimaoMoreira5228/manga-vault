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

### Running it locally for development

Two processes: the API server, and the web client talking to it through a Vite proxy.

```bash
# terminal 1 — the server, with the in-repo Lua sources loaded
PLUGINS_DIR=./plugin-repo/sources \
SECRET_KEY=$(openssl rand -hex 32) \
DATA_DIR=/tmp/manga-vault-dev \
cargo run -p manga-vault-server
```

```bash
# terminal 2 — the web client on http://localhost:5173
cd clients/web && pnpm install && pnpm dev
```

`PLUGINS_DIR` defaults to `./plugins`, which is empty in a fresh clone, so without it the app boots with **zero sources** and every list is empty. `plugin-repo/sources` holds the Lua source bundles the loader reads directly — no packing step needed. `SECRET_KEY` encrypts tracker and translation credentials; without it the server warns and those endpoints return an error. The default database is SQLite next to the binary, so `DATA_DIR` is only worth setting to keep the checkout clean.

Set `MV_API_URL` if the server is not on `127.0.0.1:8080`; the Vite dev proxy defaults to that address.

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
  --to 'sqlite://./manga-vault.db?mode=rwc'

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

Example output:

```text
entity                 legacy        new
works                       432         432  ok
chapters                  12847       12847  ok
library_entries             312         312  ok
reading_progress           8102        8102  ok
users                         3           3  ok
```


The legacy source must be MySQL or SQLite. The target can be either SQLite or Postgres, use Postgres if you plan to run the new server in hosted mode.

A SQLite target URL needs `?mode=rwc` so the file is created if it does not exist yet — without it the tool aborts with `unable to open database file`. All schema migrations for the target run automatically before any row is copied.

Every legacy `scraper` value is slugified and seeded as a `sources` row before the works reference it, which is how the legacy scraper-name string becomes a `sources.id`.

---

## Sessions and cookies

Sessions are opaque server-side rows in the `sessions` table: a random UUID token, the owning user, an optional device label, `created_at`, and `last_seen_at`. Nothing is encoded or signed into the token — the client echoes it back and the server looks it up. That is the whole revocation story: delete the row and the session is dead on the next request, with no token expiry to wait out.

### The two windows

There is one server-side window and one browser-side window, and they are not the same thing.

| Window | Value | Behaviour |
|--------|-------|-----------|
| Idle | `SESSION_TTL_DAYS` (default `7` days) | Sliding. Every authenticated request calls `touch_session`, so an active session never expires. An idle one past the window is rejected with `401 bad_credentials` **and its row is deleted on contact**. |
| Cookie `Max-Age` | the same `SESSION_TTL_DAYS`, in seconds (default `604800`) | Fixed at the moment of login or registration. It is **not** re-issued on ordinary requests. |

The consequence is the part worth knowing: `Max-Age` is a hard ceiling measured from login, while the idle window slides. A user who stays active for longer than `SESSION_TTL_DAYS` in a single browser session is signed out by the browser dropping the cookie, even though the server would still have accepted the token. Re-authenticating issues a fresh cookie. To make long-lived logins practical, raise `SESSION_TTL_DAYS` rather than relying on activity to extend the cookie.

`sessions.created_at` is stored but never used for expiry — there is no absolute maximum session lifetime independent of idle time.

### Cookie attributes

The name and attributes are derived from two settings, and one of them overrides the other:

| Setting | Cookie name | `SameSite` | `Secure` |
|---------|-------------|------------|----------|
| `SECURE_COOKIES=false`, no `CORS_ORIGINS` | `session_token` | `Lax` | no |
| `SECURE_COOKIES=true`, no `CORS_ORIGINS` | `__Host-session_token` | `Lax` | yes |
| `CORS_ORIGINS` set (any `SECURE_COOKIES`) | `__Host-session_token` | `None` | yes |

Every cookie in every row above carries `Path=/` and `HttpOnly`, and never a `Domain` attribute.

- **`SECURE_COOKIES`** decides the `__Host-` prefix. It defaults to `true` unless `BIND_ADDR` starts with `127.`, so a loopback dev server gets plain-HTTP cookies and anything network-reachable gets the prefix. Browsers reject `__Host-` cookies that arrive over plain HTTP or that carry a `Domain`, which is why the prefix is dropped rather than sent-and-ignored during local development. It also adds `Secure`, because browsers reject a `__Host-` cookie that arrives without it — set it to `false` only for plain-HTTP local work.
- **`CORS_ORIGINS`** decides `SameSite`, because a cross-site web deployment (Cloudflare Pages against a separate API host) needs `None` or the browser drops the cookie on every request. Setting it *forces* `Secure` and the `__Host-` prefix regardless of `SECURE_COOKIES`, since browsers reject `SameSite=None` without `Secure`. Same-origin deployments keep `Lax`, which is why `SECURE_COOKIES=true` alone still yields `SameSite=Lax`.

Clearing a cookie (logout, logout-everywhere) re-sends it with the same name and attributes and `Max-Age=0`.

### What else touches the session

- `GET /api/me/sessions` only lists sessions inside the idle window, and flags the caller's own token with `"current": true`.
- Logging in or registering purges that user's already-expired rows first.
- A background task enqueues a daily `CleanupExpiredData` job, which purges every expired session row regardless of owner.
- `DELETE /api/me/sessions` revokes every session including the caller's own, and clears the cookie — so a stolen token is evicted rather than left alive until its window closes.
- `POST /api/me/password` revokes every session except the caller's.

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
| `DATABASE_URL` | `sqlite://./data/manga-vault.db` | SQLite or Postgres connection. Checked at boot: the server refuses to start on an empty value, a missing `://`, or a driver it was not built with, and each failure names the accepted forms |
| `BIND_ADDR` | `127.0.0.1:8080` | API listen address |
| `DATA_DIR` | `./data` | Local data directory (downloads, repos) |
| `SECRET_KEY` | — | Encrypts tracker/translation credentials at rest. Optional; when unset the server warns at boot that those features are disabled and they return an error. Must be 64 hex chars — the server refuses to start otherwise, and refuses a key that repeats a short pattern because that encrypts nothing. Changing it makes existing credentials undecryptable |
| `SESSION_TTL_DAYS` | `7` | Idle days before a session is rejected, and the cookie's `Max-Age` in seconds. See [Sessions and cookies](#sessions-and-cookies) |
| `SECURE_COOKIES` | `true` unless `BIND_ADDR` is loopback | Marks the session cookie `__Host-`-prefixed and `Secure`, so it cannot be set over plain HTTP or scoped to a parent domain. Set it to `false` only for plain-HTTP local development |
| `ADMIN_USERNAME` | — | Grants operator access (plugin + registration management, and resetting another account's password) |
| `REGISTRATION_MODE` | `open` | `open`, `closed`, or `invite` |
| `OLLAMA_ENDPOINT` | — | Instance-wide translation via Ollama |
| `OLLAMA_MODEL` | `qwen2.5:7b` | Model used for Ollama translation |
| `TRANSLATION_ENABLED` | `true` | Kill switch for translation |
| `PLUGINS_DIR` | `./plugins` | Plugin storage path. Empty in a fresh clone; point it at `plugin-repo/sources` for the bundled Lua sources |
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
