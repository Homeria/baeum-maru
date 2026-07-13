"""임시 SQLite DB를 가리키는 Alembic 테스트 설정을 만든다."""

from pathlib import Path

from alembic.config import Config

from app.db.session import database_url

BACKEND_ROOT = Path(__file__).resolve().parents[2]


def alembic_configuration(database_file: Path) -> Config:
    configuration = Config(str(BACKEND_ROOT / "alembic.ini"))
    configuration.set_main_option(
        "sqlalchemy.url",
        database_url(database_file).render_as_string(hide_password=False),
    )
    return configuration
