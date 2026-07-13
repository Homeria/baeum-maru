"""pytest가 공유하는 임시 runtime, FastAPI application과 TestClient fixture."""

from collections.abc import Iterator
from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from app.core.runtime import RuntimePaths
from app.core.settings import AppSettings
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
