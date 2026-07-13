"""Alembic이 배움마루 metadata와 SQLite 경로를 사용하도록 구성한다."""

from __future__ import annotations

from logging.config import fileConfig

from sqlalchemy import engine_from_config, pool

from alembic import context
from app import models
from app.core.runtime import RuntimePaths
from app.db.base import Base
from app.db.session import database_url

config = context.config
if config.config_file_name is not None:
    fileConfig(config.config_file_name)

target_metadata = Base.metadata
_registered_model_modules = models.MODEL_MODULES


def configured_database_url() -> str:
    configured = config.get_main_option("sqlalchemy.url").strip()
    if configured:
        return configured

    paths = RuntimePaths.discover()
    paths.ensure_directories()
    return database_url(paths.database_file).render_as_string(hide_password=False)


def run_migrations_offline() -> None:
    context.configure(
        url=configured_database_url(),
        target_metadata=target_metadata,
        literal_binds=True,
        dialect_opts={"paramstyle": "named"},
        compare_type=True,
        render_as_batch=True,
    )

    with context.begin_transaction():
        context.run_migrations()


def run_migrations_online() -> None:
    configuration = config.get_section(config.config_ini_section, {})
    configuration["sqlalchemy.url"] = configured_database_url()
    connectable = engine_from_config(
        configuration,
        prefix="sqlalchemy.",
        poolclass=pool.NullPool,
    )

    with connectable.connect() as connection:
        context.configure(
            connection=connection,
            target_metadata=target_metadata,
            compare_type=True,
            render_as_batch=True,
        )

        with context.begin_transaction():
            context.run_migrations()


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
