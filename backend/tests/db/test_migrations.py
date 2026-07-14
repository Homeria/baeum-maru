"""빈 SQLite DB에 코드 기반 초기 schema를 반복 적용할 수 있는지 검증한다."""

from pathlib import Path

from app.core.settings import DatabaseSettings
from app.db.database import SCHEMA_VERSION, Database
from app.db.schema import TABLE_NAMES


def test_initialization_creates_tables_and_seeds_gender_codes(tmp_path: Path) -> None:
    database = Database(tmp_path / "한글 기관" / "배움마루.db", DatabaseSettings())

    database.initialize()
    database.initialize()

    with database.connection() as connection:
        actual_tables = {
            str(row[0])
            for row in connection.execute(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
            )
        }
        gender_codes = [
            tuple(row)
            for row in connection.execute(
                "SELECT code, label FROM gender_codes ORDER BY sort_order"
            )
        ]
        schema_version = connection.execute("PRAGMA user_version").fetchone()[0]

    assert actual_tables == TABLE_NAMES
    assert gender_codes == [
        ("male", "남성"),
        ("female", "여성"),
        ("unknown", "미상"),
    ]
    assert schema_version == SCHEMA_VERSION
