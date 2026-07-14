"""실제 sqlite3 DDL이 적용된 테스트 DB fixture."""

from pathlib import Path

import pytest

from app.core.settings import DatabaseSettings
from app.db.database import Database


@pytest.fixture
def initialized_database(tmp_path: Path) -> Database:
    database_file = tmp_path / "한글 기관" / "schema-contract" / "배움마루.db"
    database = Database(database_file, DatabaseSettings())
    database.initialize()
    return database
