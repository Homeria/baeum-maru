"""애플리케이션 시작 시 SQLite schema를 최신 Alembic revision으로 올린다."""

from pathlib import Path

from alembic.config import Config

from alembic import command
from app.db.session import database_url

BACKEND_ROOT = Path(__file__).resolve().parents[2]


def alembic_configuration(database_file: Path) -> Config:
    configuration = Config(str(BACKEND_ROOT / "alembic.ini"))
    configuration.attributes["configure_logger"] = False
    configuration.set_main_option(
        "sqlalchemy.url",
        database_url(database_file).render_as_string(hide_password=False),
    )
    return configuration


def upgrade_database(database_file: Path) -> None:
    """빈 DB 생성과 이후 revision 적용을 모두 포함해 Alembic head로 이동한다."""
    database_file.parent.mkdir(parents=True, exist_ok=True)
    command.upgrade(alembic_configuration(database_file), "head")
