"""SQLite 연결 설정과 명시적 sqlite3 transaction을 검증한다."""

import sqlite3
from pathlib import Path

import pytest

from app.core.settings import DatabaseSettings
from app.db.database import Database


@pytest.fixture
def database(tmp_path: Path) -> Database:
    database_file = tmp_path / "한글 사용자" / "운영 자료" / "배움마루.db"
    return Database(database_file, DatabaseSettings(busy_timeout_ms=7_000))


def test_database_preserves_portable_file_path(database: Database, tmp_path: Path) -> None:
    expected = (tmp_path / "한글 사용자" / "운영 자료" / "배움마루.db").resolve()
    assert database.database_file == expected


def test_connection_applies_required_sqlite_pragmas(database: Database) -> None:
    with database.connection() as connection:
        foreign_keys = connection.execute("PRAGMA foreign_keys").fetchone()[0]
        busy_timeout = connection.execute("PRAGMA busy_timeout").fetchone()[0]
        journal_mode = connection.execute("PRAGMA journal_mode").fetchone()[0]
        synchronous = connection.execute("PRAGMA synchronous").fetchone()[0]

    assert foreign_keys == 1
    assert busy_timeout == 7_000
    assert journal_mode == "wal"
    assert synchronous == 1


def test_foreign_key_constraint_is_enforced(database: Database) -> None:
    with database.transaction() as connection:
        connection.execute("CREATE TABLE parents (id INTEGER PRIMARY KEY)")
        connection.execute(
            """
            CREATE TABLE children (
                id INTEGER PRIMARY KEY,
                parent_id INTEGER NOT NULL REFERENCES parents(id)
            )
            """
        )

    with pytest.raises(sqlite3.IntegrityError), database.transaction() as connection:
        connection.execute("INSERT INTO children (id, parent_id) VALUES (1, 999)")


def test_uncommitted_connection_is_rolled_back_on_close(database: Database) -> None:
    with database.transaction() as connection:
        connection.execute("CREATE TABLE samples (id INTEGER PRIMARY KEY)")

    with database.connection() as connection:
        connection.execute("INSERT INTO samples (id) VALUES (1)")

    with database.connection() as connection:
        count = connection.execute("SELECT COUNT(*) FROM samples").fetchone()[0]
    assert count == 0


def test_transaction_commits_or_rolls_back_as_one_unit(database: Database) -> None:
    with database.transaction() as connection:
        connection.execute("CREATE TABLE samples (id INTEGER PRIMARY KEY)")

    with database.transaction() as connection:
        connection.execute("INSERT INTO samples (id) VALUES (1)")

    with pytest.raises(sqlite3.IntegrityError), database.transaction() as connection:
        connection.execute("INSERT INTO samples (id) VALUES (2)")
        connection.execute("INSERT INTO samples (id) VALUES (1)")

    with database.connection() as connection:
        ids = [row[0] for row in connection.execute("SELECT id FROM samples ORDER BY id")]
    assert ids == [1]
