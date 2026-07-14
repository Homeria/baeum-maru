"""FastAPI lifespan, health endpoint와 OpenAPI 기본 계약을 검증한다."""

import logging
import os
import subprocess
import sys
from logging.handlers import RotatingFileHandler
from pathlib import Path

from fastapi import FastAPI
from fastapi.testclient import TestClient

from app.db.schema import TABLE_NAMES


def test_lifespan_prepares_runtime_database_and_application_state(
    api_app: FastAPI, api_client: TestClient
) -> None:
    paths = api_app.state.runtime_paths

    assert paths.root.is_dir()
    assert paths.database_file.is_file()
    assert paths.application_log_file.is_file()
    assert api_app.state.settings.environment == "test"
    assert any(
        isinstance(handler, RotatingFileHandler)
        and Path(handler.baseFilename) == paths.application_log_file
        for handler in logging.getLogger().handlers
    )
    with api_app.state.database.connection() as connection:
        table_names = {
            str(row[0])
            for row in connection.execute(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
            )
        }
    assert table_names == TABLE_NAMES


def test_health_endpoint_is_versioned(api_client: TestClient) -> None:
    response = api_client.get("/api/v1/health")

    assert response.status_code == 200
    assert response.json() == {"status": "ok", "service": "baeum-maru"}


def test_openapi_exposes_metadata_and_health_path(api_client: TestClient) -> None:
    response = api_client.get("/api/v1/openapi.json")

    assert response.status_code == 200
    document = response.json()
    assert document["info"]["title"] == "배움마루 API"
    assert document["info"]["version"] == "0.1.0"
    assert "/api/v1/health" in document["paths"]
    assert "/api/v1/openapi.json" not in document["paths"]


def test_module_import_does_not_create_runtime_files(tmp_path: Path) -> None:
    runtime_root = tmp_path / "import-only-runtime"
    backend_root = Path(__file__).resolve().parents[2]
    environment = os.environ | {"BAEUM_MARU_RUNTIME_DIR": str(runtime_root)}

    subprocess.run(
        [sys.executable, "-c", "import app.main"],
        cwd=backend_root,
        env=environment,
        check=True,
        capture_output=True,
        text=True,
    )

    assert not runtime_root.exists()
