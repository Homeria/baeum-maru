"""SQLite engine과 request 단위 Session factory를 생성한다."""

from __future__ import annotations

import sqlite3
from pathlib import Path

from sqlalchemy import URL, Engine, event
from sqlalchemy import create_engine as sqlalchemy_create_engine
from sqlalchemy.orm import Session, sessionmaker

from app.core.settings import DatabaseSettings


def database_url(database_file: Path) -> URL:
    """특수문자와 Windows 경로를 안전하게 처리하는 SQLite URL을 만든다."""
    return URL.create("sqlite+pysqlite", database=str(database_file.resolve()))


def create_sqlite_engine(database_file: Path, settings: DatabaseSettings) -> Engine:
    """파일 기반 SQLite engine을 만들고 모든 연결에 운영 PRAGMA를 적용한다."""
    database_file.parent.mkdir(parents=True, exist_ok=True)
    engine = sqlalchemy_create_engine(
        database_url(database_file),
        connect_args={
            "check_same_thread": False,
            "timeout": settings.busy_timeout_ms / 1000,
        },
        echo=settings.echo_sql,
        pool_pre_ping=True,
    )

    def configure_connection(
        dbapi_connection: sqlite3.Connection,
        _connection_record: object,
    ) -> None:
        cursor = dbapi_connection.cursor()
        try:
            cursor.execute("PRAGMA foreign_keys = ON")
            cursor.execute(f"PRAGMA busy_timeout = {settings.busy_timeout_ms}")
            cursor.execute("PRAGMA journal_mode = WAL")
            cursor.execute("PRAGMA synchronous = NORMAL")
        finally:
            cursor.close()

    event.listen(engine, "connect", configure_connection)
    return engine


def create_session_factory(engine: Engine) -> sessionmaker[Session]:
    """자동 commit 없이 명시적 transaction을 사용하는 Session factory를 만든다."""
    return sessionmaker(bind=engine, autoflush=False, expire_on_commit=False)
