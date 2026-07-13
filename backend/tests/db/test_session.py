"""SQLite 연결 설정과 명시적 Session transaction을 검증한다."""

from collections.abc import Iterator
from pathlib import Path

import pytest
from sqlalchemy import Engine, text
from sqlalchemy.exc import IntegrityError

from app.core.settings import DatabaseSettings
from app.db.session import create_session_factory, create_sqlite_engine, database_url


@pytest.fixture
def engine(tmp_path: Path) -> Iterator[Engine]:
    database_file = tmp_path / "한글 사용자" / "운영 자료" / "배움마루.db"
    value = create_sqlite_engine(database_file, DatabaseSettings(busy_timeout_ms=7_000))
    try:
        yield value
    finally:
        value.dispose()


def test_database_url_preserves_portable_file_path(tmp_path: Path) -> None:
    database_file = tmp_path / "기관 자료" / "배움마루.db"

    url = database_url(database_file)

    assert url.drivername == "sqlite+pysqlite"
    assert url.database == str(database_file.resolve())


def test_engine_applies_required_sqlite_pragmas(engine: Engine) -> None:
    with engine.connect() as connection:
        foreign_keys = connection.scalar(text("PRAGMA foreign_keys"))
        busy_timeout = connection.scalar(text("PRAGMA busy_timeout"))
        journal_mode = connection.scalar(text("PRAGMA journal_mode"))
        synchronous = connection.scalar(text("PRAGMA synchronous"))

    assert foreign_keys == 1
    assert busy_timeout == 7_000
    assert journal_mode == "wal"
    assert synchronous == 1


def test_foreign_key_constraint_is_enforced(engine: Engine) -> None:
    with engine.begin() as connection:
        connection.execute(text("CREATE TABLE parents (id INTEGER PRIMARY KEY)"))
        connection.execute(
            text(
                "CREATE TABLE children ("
                "id INTEGER PRIMARY KEY, "
                "parent_id INTEGER NOT NULL REFERENCES parents(id)"
                ")"
            )
        )

    with pytest.raises(IntegrityError), engine.begin() as connection:
        connection.execute(text("INSERT INTO children (id, parent_id) VALUES (1, 999)"))


def test_session_does_not_commit_implicitly(engine: Engine) -> None:
    with engine.begin() as connection:
        connection.execute(text("CREATE TABLE samples (id INTEGER PRIMARY KEY)"))

    factory = create_session_factory(engine)
    with factory() as session:
        session.execute(text("INSERT INTO samples (id) VALUES (1)"))

    with factory() as session:
        assert session.scalar(text("SELECT COUNT(*) FROM samples")) == 0
        session.execute(text("INSERT INTO samples (id) VALUES (2)"))
        session.commit()

    with factory() as session:
        assert session.scalar(text("SELECT COUNT(*) FROM samples")) == 1
