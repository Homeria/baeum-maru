"""공통 오류 response와 request ID correlation 계약을 검증한다."""

from fastapi import FastAPI, HTTPException, Query
from fastapi.testclient import TestClient

from app.core.exceptions import ConflictError


def test_request_id_is_preserved_when_valid(api_client: TestClient) -> None:
    response = api_client.get("/api/v1/health", headers={"X-Request-ID": "reception-123"})

    assert response.headers["X-Request-ID"] == "reception-123"


def test_invalid_request_id_is_replaced(api_client: TestClient) -> None:
    invalid_request_id = "request id with spaces"
    response = api_client.get("/api/v1/health", headers={"X-Request-ID": invalid_request_id})

    request_id = response.headers["X-Request-ID"]
    assert request_id != invalid_request_id
    assert len(request_id) == 36


def test_validation_error_has_field_details_and_request_id(
    api_app: FastAPI, api_client: TestClient
) -> None:
    def validated_endpoint(value: int = Query(ge=1)) -> dict[str, int]:
        return {"value": value}

    api_app.add_api_route("/api/v1/test/validation", validated_endpoint, methods=["GET"])

    response = api_client.get(
        "/api/v1/test/validation?value=0",
        headers={"X-Request-ID": "validation-1"},
    )

    assert response.status_code == 422
    assert response.json() == {
        "error": {
            "code": "validation_error",
            "message": "요청 값이 올바르지 않습니다.",
            "request_id": "validation-1",
            "details": [
                {
                    "field": "query.value",
                    "message": "Input should be greater than or equal to 1",
                    "type": "greater_than_equal",
                }
            ],
        }
    }


def test_application_conflict_maps_to_409(api_app: FastAPI, api_client: TestClient) -> None:
    def conflicting_endpoint() -> None:
        raise ConflictError("stale_version", "다른 사용자가 먼저 수정했습니다.")

    api_app.add_api_route("/api/v1/test/conflict", conflicting_endpoint, methods=["POST"])

    response = api_client.post("/api/v1/test/conflict", headers={"X-Request-ID": "conflict-1"})

    assert response.status_code == 409
    assert response.json()["error"] == {
        "code": "stale_version",
        "message": "다른 사용자가 먼저 수정했습니다.",
        "request_id": "conflict-1",
        "details": [],
    }


def test_unknown_route_uses_common_error_shape(api_client: TestClient) -> None:
    response = api_client.get("/api/v1/does-not-exist")

    assert response.status_code == 404
    assert response.json()["error"]["code"] == "http_error"
    assert response.json()["error"]["request_id"] == response.headers["X-Request-ID"]


def test_http_error_preserves_protocol_headers(api_app: FastAPI, api_client: TestClient) -> None:
    def unauthorized_endpoint() -> None:
        raise HTTPException(
            status_code=401,
            detail="인증이 필요합니다.",
            headers={"WWW-Authenticate": "Bearer"},
        )

    api_app.add_api_route("/api/v1/test/unauthorized", unauthorized_endpoint, methods=["GET"])

    response = api_client.get("/api/v1/test/unauthorized")

    assert response.status_code == 401
    assert response.headers["WWW-Authenticate"] == "Bearer"
    assert response.json()["error"]["message"] == "인증이 필요합니다."


def test_unexpected_error_hides_internal_message(api_app: FastAPI) -> None:
    def failing_endpoint() -> None:
        raise RuntimeError("database password leaked")

    api_app.add_api_route("/api/v1/test/failure", failing_endpoint, methods=["GET"])

    with TestClient(api_app, raise_server_exceptions=False) as client:
        response = client.get("/api/v1/test/failure", headers={"X-Request-ID": "failure-1"})

    assert response.status_code == 500
    assert response.json()["error"] == {
        "code": "internal_error",
        "message": "서버에서 요청을 처리하지 못했습니다.",
        "request_id": "failure-1",
        "details": [],
    }
    assert "database password leaked" not in response.text
