import datetime
import json
import os
import sqlite3
import urllib.request
import uuid

API = os.environ["E2E_API_URL"]
DB_PATH = os.environ["E2E_DB_PATH"]
PASSWORD = "e2e-password-1234"
USERS = (
    "admin",
    "library_reader",
    "reader",
    "searcher",
    "settings_viewer",
    "shortcuts_user",
)
MANGA_COUNT = 40
NOVEL_COUNT = 20
CHAPTER_COUNT = 5
NOW = datetime.datetime.now(datetime.timezone.utc).replace(microsecond=123456)


def entry_stamp(index: int) -> str:
    return (NOW - datetime.timedelta(hours=3, minutes=index)).isoformat()


def work_stamp(index: int) -> str:
    return (NOW - datetime.timedelta(minutes=5) + datetime.timedelta(seconds=2 * index)).isoformat()


def dumps(value) -> str:
    return json.dumps(value, separators=(",", ":"))


def register(username: str) -> bytes:
    request = urllib.request.Request(
        f"{API}/api/auth/register",
        data=json.dumps({"username": username, "password": PASSWORD}).encode(),
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request) as response:
        session = json.load(response)
    return uuid.UUID(session["user_id"]).bytes


def manga_work(number: int) -> dict:
    return {
        "kind": "manga",
        "source_id": "mv_e2e_manga",
        "remote_url": f"e2e-manga://work/{number}",
        "title": f"Manga Seed {number:02d}",
        "alternative_names": [],
        "authors": ["Seed Author"],
        "artists": ["Seed Artist"],
        "status": "ongoing",
        "release_date": "2026-01-01",
        "description": f"Seeded manga fixture number {number} for the end-to-end suite.",
        "genres": ["seed", "action"],
    }


def novel_work(number: int) -> dict:
    return {
        "kind": "novel",
        "source_id": "mv_e2e_novel",
        "remote_url": f"e2e-novel://work/{number}",
        "title": f"Novel Seed {number:02d}",
        "alternative_names": [f"Novel Seed {number:02d} Alias"],
        "authors": ["Seed Writer"],
        "artists": [],
        "status": "ongoing",
        "release_date": "2026-01-01",
        "description": f"Seeded novel fixture number {number} for the end-to-end suite.",
        "genres": ["seed", "fantasy"],
    }


def insert_work(connection, work: dict, index: int) -> bytes:
    work_id = uuid.uuid4().bytes
    connection.execute(
        "INSERT INTO works (id, kind, source_id, remote_url, title, cover_url, alternative_names,"
        " authors, artists, status, release_date, description, genres, created_at, updated_at)"
        " VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            work_id,
            work["kind"],
            work["source_id"],
            work["remote_url"],
            work["title"],
            None,
            dumps(work["alternative_names"]),
            dumps(work["authors"]),
            dumps(work["artists"]),
            work["status"],
            work["release_date"],
            work["description"],
            dumps(work["genres"]),
            entry_stamp(index),
            work_stamp(index),
        ),
    )
    return work_id


def insert_chapters(connection, work_id: bytes, work: dict) -> list:
    chapter_ids = []
    for number in range(1, CHAPTER_COUNT + 1):
        chapter_id = uuid.uuid4().bytes
        connection.execute(
            "INSERT INTO chapters (id, work_id, title, remote_url, sort_index, content_kind,"
            " scanlation_group, released_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                chapter_id,
                work_id,
                f"Chapter {number}",
                f"{work['remote_url']}/chapter/{number}",
                number - 1,
                "images" if work["kind"] == "manga" else "html",
                "Seed Scans" if work["kind"] == "manga" else None,
                f"2026-01-{number:02d}T00:00:00+00:00",
                entry_stamp(0),
            ),
        )
        chapter_ids.append(chapter_id)
    return chapter_ids


def insert_entry(connection, user_id: bytes, work_id: bytes, index: int) -> None:
    connection.execute(
        "INSERT INTO library_entries (id, user_id, work_id, category_id, created_at, updated_at,"
        " removed_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        (uuid.uuid4().bytes, user_id, work_id, None, entry_stamp(index), entry_stamp(index), None),
    )


def insert_progress(connection, user_id: bytes, work_id: bytes, chapter_ids) -> None:
    stamp = (NOW - datetime.timedelta(days=1)).isoformat()
    for chapter_id in chapter_ids:
        connection.execute(
            "INSERT INTO reading_progress (id, user_id, work_id, chapter_id, read_at, updated_at,"
            " removed_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            (uuid.uuid4().bytes, user_id, work_id, chapter_id, stamp, stamp, None),
        )


def main() -> None:
    users = {name: register(name) for name in USERS}
    connection = sqlite3.connect(DB_PATH)
    works = [manga_work(number) for number in range(1, MANGA_COUNT + 1)]
    works += [novel_work(number) for number in range(1, NOVEL_COUNT + 1)]
    work_ids = {}
    chapters = {}
    for index, work in enumerate(works):
        work_id = insert_work(connection, work, index)
        work_ids[work["remote_url"]] = work_id
        chapters[work["remote_url"]] = insert_chapters(connection, work_id, work)
    for index, work in enumerate(works):
        insert_entry(connection, users["library_reader"], work_ids[work["remote_url"]], index)
    for url in ("e2e-manga://work/7", "e2e-novel://work/7"):
        insert_entry(connection, users["reader"], work_ids[url], 0)
    insert_progress(connection, users["library_reader"], work_ids["e2e-manga://work/1"], chapters["e2e-manga://work/1"][:2])
    insert_progress(connection, users["library_reader"], work_ids["e2e-manga://work/2"], chapters["e2e-manga://work/2"][:4])
    connection.commit()
    connection.close()
    print(f"seeded {len(works)} works, {len(works) * CHAPTER_COUNT} chapters", flush=True)


main()
