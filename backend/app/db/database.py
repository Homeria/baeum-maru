"""표준 sqlite3 연결, schema 초기화와 transaction 경계를 제공한다."""

from __future__ import annotations

import logging
import sqlite3
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

from app.core.settings import DatabaseSettings
from app.db.schema import SCHEMA_STATEMENTS, SEED_STATEMENTS

SCHEMA_VERSION = 1


class Database:
    """SQLite 파일에 대한 연결 설정을 한 곳에서 관리한다."""

    def __init__(self, database_file: Path, settings: DatabaseSettings) -> None:
        self.database_file = database_file.resolve()
        self.settings = settings

    def connect(self) -> sqlite3.Connection:
        """운영 PRAGMA와 Row factory가 적용된 새 연결을 반환한다."""
        self.database_file.parent.mkdir(parents=True, exist_ok=True)
        connection = sqlite3.connect(
            self.database_file,
            timeout=self.settings.busy_timeout_ms / 1000,
            check_same_thread=False,
        )
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA foreign_keys = ON")
        connection.execute(f"PRAGMA busy_timeout = {self.settings.busy_timeout_ms}")
        connection.execute("PRAGMA journal_mode = WAL")
        connection.execute("PRAGMA synchronous = NORMAL")
        if self.settings.echo_sql:
            sql_logger = logging.getLogger("app.db.sql")
            connection.set_trace_callback(sql_logger.debug)
        return connection

    def initialize(self) -> None:
        """빈 DB에 현재 schema와 필수 기준 데이터를 생성한다."""
        with self.transaction() as connection:
            for statement in SCHEMA_STATEMENTS:
                connection.execute(statement)
            for statement in SEED_STATEMENTS:
                connection.execute(statement)
            connection.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")

    @contextmanager
    def connection(self) -> Iterator[sqlite3.Connection]:
        """요청이나 조회가 사용할 연결을 열고 남은 transaction을 정리한다."""
        connection = self.connect()
        try:
            yield connection
        finally:
            if connection.in_transaction:
                connection.rollback()
            connection.close()

    @contextmanager
    def transaction(self) -> Iterator[sqlite3.Connection]:
        """여러 repository 작업을 하나의 commit 또는 rollback으로 묶는다."""
        with self.connection() as connection:
            connection.execute("BEGIN")
            try:
                yield connection
            except BaseException:
                connection.rollback()
                raise
            else:
                connection.commit()
