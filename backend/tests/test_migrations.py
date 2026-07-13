from pathlib import Path
from typing import cast

from alembic.config import Config
from sqlalchemy import create_engine, inspect, text

from alembic import command
from app.db.session import database_url
from tests.test_db_metadata import EXPECTED_TABLES

BACKEND_ROOT = Path(__file__).resolve().parents[1]


def test_initial_migration_matches_metadata_and_seeds_reference_data(tmp_path: Path) -> None:
    database_file = tmp_path / "migration" / "baeum-maru.db"
    database_file.parent.mkdir(parents=True)
    configuration = _alembic_configuration(database_file)

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


def _alembic_configuration(database_file: Path) -> Config:
    configuration = Config(str(BACKEND_ROOT / "alembic.ini"))
    configuration.set_main_option(
        "sqlalchemy.url",
        database_url(database_file).render_as_string(hide_password=False),
    )
    return configuration
