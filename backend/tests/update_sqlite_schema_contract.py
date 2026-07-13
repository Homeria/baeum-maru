from pathlib import Path
from tempfile import TemporaryDirectory

from alembic.config import Config
from sqlalchemy import create_engine, inspect

from alembic import command
from app.db.session import database_url
from tests.schema_contract import build_sqlite_schema_contract, write_sqlite_schema_contract

BACKEND_ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    with TemporaryDirectory() as temporary_directory:
        database_file = Path(temporary_directory) / "schema-contract.db"
        configuration = Config(str(BACKEND_ROOT / "alembic.ini"))
        configuration.set_main_option(
            "sqlalchemy.url",
            database_url(database_file).render_as_string(hide_password=False),
        )
        command.upgrade(configuration, "head")

        engine = create_engine(database_url(database_file))
        try:
            contract = build_sqlite_schema_contract(inspect(engine))
        finally:
            engine.dispose()

    write_sqlite_schema_contract(contract)


if __name__ == "__main__":
    main()
