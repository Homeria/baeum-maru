"""request scope DB connection과 pagination 의존성을 검증한다."""

import sqlite3
from typing import Annotated

from fastapi import Depends, FastAPI
from fastapi.testclient import TestClient

from app.api.dependencies import get_db, get_pagination
from app.schemas.common import PageMetadata, PaginationParams


def test_get_db_yields_configured_request_session(api_app: FastAPI, api_client: TestClient) -> None:
    def database_probe(
        connection: Annotated[sqlite3.Connection, Depends(get_db)],
    ) -> dict[str, int]:
        return {
            "foreign_keys": int(connection.execute("PRAGMA foreign_keys").fetchone()[0]),
            "busy_timeout": int(connection.execute("PRAGMA busy_timeout").fetchone()[0]),
        }

    api_app.add_api_route("/api/v1/test/database", database_probe, methods=["GET"])

    response = api_client.get("/api/v1/test/database")

    assert response.status_code == 200
    assert response.json() == {"foreign_keys": 1, "busy_timeout": 5000}


def test_pagination_dependency_and_metadata(api_app: FastAPI, api_client: TestClient) -> None:
    def pagination_probe(
        pagination: Annotated[PaginationParams, Depends(get_pagination)],
    ) -> dict[str, int]:
        metadata = PageMetadata.from_total(pagination, total_items=45)
        return {
            "page": pagination.page,
            "page_size": pagination.page_size,
            "offset": pagination.offset,
            "total_pages": metadata.total_pages,
        }

    api_app.add_api_route("/api/v1/test/pagination", pagination_probe, methods=["GET"])

    response = api_client.get("/api/v1/test/pagination?page=2&page_size=20")

    assert response.status_code == 200
    assert response.json() == {"page": 2, "page_size": 20, "offset": 20, "total_pages": 3}


def test_pagination_rejects_values_outside_contract(
    api_app: FastAPI, api_client: TestClient
) -> None:
    def pagination_probe(
        pagination: Annotated[PaginationParams, Depends(get_pagination)],
    ) -> dict[str, int]:
        return {"page": pagination.page}

    api_app.add_api_route("/api/v1/test/pagination", pagination_probe, methods=["GET"])

    response = api_client.get("/api/v1/test/pagination?page=0&page_size=101")

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "validation_error"
