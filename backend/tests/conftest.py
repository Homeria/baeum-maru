"""pytest가 공유하는 임시 runtime, FastAPI application과 TestClient fixture."""

from collections.abc import Iterator
from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from sqlalchemy.orm import Session, sessionmaker

from alembic import command
from app.core.runtime import RuntimePaths
from app.core.settings import AppSettings, DatabaseSettings
from app.db.migrations import alembic_configuration
from app.db.session import create_session_factory, create_sqlite_engine
from app.main import create_app


@pytest.fixture
def api_app(tmp_path: Path) -> FastAPI:
    paths = RuntimePaths.discover(tmp_path / "한글 기관" / "API runtime")
    settings = AppSettings(environment="test")
    return create_app(runtime_paths=paths, settings=settings)


@pytest.fixture
def api_client(api_app: FastAPI) -> Iterator[TestClient]:
    with TestClient(api_app) as client:
        yield client


@pytest.fixture
def migrated_session_factory(tmp_path: Path) -> Iterator[sessionmaker[Session]]:
    database_file = tmp_path / "한글 기관" / "service-tests" / "배움마루.db"
    database_file.parent.mkdir(parents=True)
    command.upgrade(alembic_configuration(database_file), "head")
    engine = create_sqlite_engine(database_file, DatabaseSettings())
    try:
        yield create_session_factory(engine)
    finally:
        engine.dispose()
