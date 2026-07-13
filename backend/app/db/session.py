from __future__ import annotations

import sqlite3
from dataclasses import dataclass
from pathlib import Path

from sqlalchemy import URL, Engine, create_engine, event
from sqlalchemy.orm import Session, sessionmaker

SQLITE_BUSY_TIMEOUT_MS = 5_000


@dataclass(frozen=True, slots=True)
class Database:
    engine: Engine
    session_factory: sessionmaker[Session]

    def dispose(self) -> None:
        self.engine.dispose()


def database_url(database_file: Path) -> URL:
    return URL.create("sqlite+pysqlite", database=str(database_file.resolve()))


def create_database(database_file: Path) -> Database:
    database_file.parent.mkdir(parents=True, exist_ok=True)
    engine = create_engine(database_url(database_file), pool_pre_ping=True)
    event.listen(engine, "connect", _configure_sqlite_connection)
    factory = sessionmaker(bind=engine, autoflush=False, expire_on_commit=False)
    return Database(engine=engine, session_factory=factory)


def _configure_sqlite_connection(
    dbapi_connection: sqlite3.Connection,
    _connection_record: object,
) -> None:
    cursor = dbapi_connection.cursor()
    try:
        cursor.execute("PRAGMA foreign_keys = ON")
        cursor.execute(f"PRAGMA busy_timeout = {SQLITE_BUSY_TIMEOUT_MS}")
        cursor.execute("PRAGMA journal_mode = WAL")
    finally:
        cursor.close()
