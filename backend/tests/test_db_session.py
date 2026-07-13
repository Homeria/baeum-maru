from pathlib import Path

from sqlalchemy import text

from app.db.session import SQLITE_BUSY_TIMEOUT_MS, create_database


def test_sqlite_engine_prepares_portable_database_connection(tmp_path: Path) -> None:
    database_file = tmp_path / "한글 사용자" / "운영 자료" / "baeum-maru.db"
    database = create_database(database_file)

    try:
        with database.engine.connect() as connection:
            foreign_keys = connection.scalar(text("PRAGMA foreign_keys"))
            busy_timeout = connection.scalar(text("PRAGMA busy_timeout"))
            journal_mode = connection.scalar(text("PRAGMA journal_mode"))

        assert database_file.is_file()
        assert foreign_keys == 1
        assert busy_timeout == SQLITE_BUSY_TIMEOUT_MS
        assert journal_mode == "wal"
    finally:
        database.dispose()
