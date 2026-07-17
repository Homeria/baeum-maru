"""함수 기반 sqlite3 연결, schema 초기화와 transaction 경계를 제공한다."""

from __future__ import annotations

import logging
import sqlite3
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

from app.core.runtime import RuntimePaths
from app.core.settings import DatabaseSettings, load_settings
from app.db.schema import SCHEMA_STATEMENTS, SEED_STATEMENTS

SCHEMA_VERSION = 1


def _resolve_database_options(
    database_file: Path | None,
    settings: DatabaseSettings | None,
) -> tuple[Path, DatabaseSettings]:
    runtime_paths = RuntimePaths.discover()
    resolved_file = (database_file or runtime_paths.database_file).resolve()
    resolved_settings = settings
    if resolved_settings is None:
        resolved_settings = (
            load_settings(runtime_paths).database if database_file is None else DatabaseSettings()
        )
    return resolved_file, resolved_settings


@contextmanager
def get_db_connection(
    database_file: Path | None = None,
    settings: DatabaseSettings | None = None,
) -> Iterator[sqlite3.Connection]:
    """운영 DB 또는 전달받은 테스트 DB에 연결하고 사용 후 닫는다."""
    resolved_file, resolved_settings = _resolve_database_options(database_file, settings)
    resolved_file.parent.mkdir(parents=True, exist_ok=True)
    connection = sqlite3.connect(
        resolved_file,
        timeout=resolved_settings.busy_timeout_ms / 1000,
        check_same_thread=False,
    )
    connection.row_factory = sqlite3.Row
    connection.execute("PRAGMA foreign_keys = ON")
    connection.execute(f"PRAGMA busy_timeout = {resolved_settings.busy_timeout_ms}")
    connection.execute("PRAGMA journal_mode = WAL")
    connection.execute("PRAGMA synchronous = NORMAL")
    if resolved_settings.echo_sql:
        connection.set_trace_callback(logging.getLogger("app.db.sql").debug)

    try:
        yield connection
    finally:
        if connection.in_transaction:
            connection.rollback()
        connection.close()


@contextmanager
def transaction(
    database_file: Path | None = None,
    settings: DatabaseSettings | None = None,
) -> Iterator[sqlite3.Connection]:
    """Repository 작업을 commit하거나 예외 발생 시 rollback한다."""
    with get_db_connection(database_file, settings) as connection:
        connection.execute("BEGIN")
        try:
            yield connection
        except BaseException:
            connection.rollback()
            raise
        else:
            connection.commit()


def initialize_database(
    database_file: Path | None = None,
    settings: DatabaseSettings | None = None,
) -> None:
    """빈 DB에 현재 schema와 필수 기준 데이터를 생성한다."""
    with transaction(database_file, settings) as connection:
        for statement in SCHEMA_STATEMENTS:
            connection.execute(statement)
        for statement in SEED_STATEMENTS:
            connection.execute(statement)
        connection.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")
