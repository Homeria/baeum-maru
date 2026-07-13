"""실제 Alembic migration이 적용된 SQLite 테스트 DB fixture."""

from collections.abc import Iterator
from pathlib import Path

import pytest
from sqlalchemy import Engine

from alembic import command
from app.core.settings import DatabaseSettings
from app.db.session import create_sqlite_engine
from tests.db.alembic_helpers import alembic_configuration


@pytest.fixture
def migrated_engine(tmp_path: Path) -> Iterator[Engine]:
    database_file = tmp_path / "한글 기관" / "schema-contract" / "배움마루.db"
    database_file.parent.mkdir(parents=True)
    command.upgrade(alembic_configuration(database_file), "head")

    engine = create_sqlite_engine(database_file, DatabaseSettings())
    try:
        yield engine
    finally:
        engine.dispose()
