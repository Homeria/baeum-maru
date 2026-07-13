"""빈 SQLite DB에 초기 Alembic revision을 적용하고 되돌릴 수 있는지 검증한다."""

from pathlib import Path
from typing import cast

from sqlalchemy import create_engine, inspect, text

from alembic import command
from app.db.migrations import alembic_configuration
from app.db.session import database_url
from tests.db.test_metadata import EXPECTED_TABLES


def test_initial_migration_matches_metadata_and_seeds_gender_codes(tmp_path: Path) -> None:
    database_file = tmp_path / "한글 기관" / "migration" / "배움마루.db"
    database_file.parent.mkdir(parents=True)
    configuration = alembic_configuration(database_file)

    command.upgrade(configuration, "head")
    command.check(configuration)

    engine = create_engine(database_url(database_file))
    try:
        with engine.connect() as connection:
            actual_tables = set(inspect(connection).get_table_names())
            gender_codes = cast(
                list[tuple[str, str]],
                connection.execute(text("SELECT code, label FROM gender_codes ORDER BY sort_order"))
                .tuples()
                .all(),
            )

        assert actual_tables == EXPECTED_TABLES | {"alembic_version"}
        assert gender_codes == [
            ("male", "남성"),
            ("female", "여성"),
            ("unknown", "미상"),
        ]
    finally:
        engine.dispose()

    command.downgrade(configuration, "base")
    downgraded_engine = create_engine(database_url(database_file))
    try:
        remaining_tables = set(inspect(downgraded_engine).get_table_names())
        assert not remaining_tables.intersection(EXPECTED_TABLES)
    finally:
        downgraded_engine.dispose()
