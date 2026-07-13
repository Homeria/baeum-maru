"""인증 WebSocket의 연결, heartbeat와 resource event 계약을 검증한다."""

from collections.abc import Iterator
from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from starlette import status
from starlette.websockets import WebSocketDisconnect

from app.api.dependencies import SESSION_COOKIE_NAME
from app.core.runtime import RuntimePaths
from app.core.settings import AppSettings, RealtimeSettings
from app.main import create_app
from app.services.realtime_service import ResourceEvent, publish_committed_events

WEBSOCKET_PATH = "/api/v1/events/ws"
SAME_ORIGIN_HEADERS = {"origin": "http://testserver"}


@pytest.fixture
def realtime_app(tmp_path: Path) -> FastAPI:
    paths = RuntimePaths.discover(tmp_path / "한글 기관" / "realtime runtime")

    def verify_session(token: str) -> str | None:
        return "session-1" if token == "valid-token" else None

    return create_app(
        runtime_paths=paths,
        settings=AppSettings(environment="test"),
        realtime_session_verifier=verify_session,
    )


@pytest.fixture
def realtime_client(realtime_app: FastAPI) -> Iterator[TestClient]:
    with TestClient(realtime_app) as client:
        client.cookies.set(SESSION_COOKIE_NAME, "valid-token")
        yield client


def test_realtime_rejects_missing_session_cookie(api_client: TestClient) -> None:
    with (
        pytest.raises(WebSocketDisconnect) as closed,
        api_client.websocket_connect(WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS),
    ):
        pass

    assert closed.value.code == status.WS_1008_POLICY_VIOLATION


def test_realtime_rejects_invalid_session_cookie(realtime_app: FastAPI) -> None:
    with TestClient(realtime_app) as client:
        client.cookies.set(SESSION_COOKIE_NAME, "invalid-token")
        with (
            pytest.raises(WebSocketDisconnect) as closed,
            client.websocket_connect(WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS),
        ):
            pass

    assert closed.value.code == status.WS_1008_POLICY_VIOLATION


def test_realtime_rejects_cross_origin_cookie(realtime_client: TestClient) -> None:
    with (
        pytest.raises(WebSocketDisconnect) as closed,
        realtime_client.websocket_connect(
            WEBSOCKET_PATH,
            headers={"origin": "https://malicious.example"},
        ),
    ):
        pass

    assert closed.value.code == status.WS_1008_POLICY_VIOLATION


def test_each_connection_requires_reconciliation(realtime_client: TestClient) -> None:
    ready_messages: list[dict[str, object]] = []

    for _ in range(2):
        with realtime_client.websocket_connect(
            WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS
        ) as websocket:
            ready_messages.append(websocket.receive_json())

    assert [message["type"] for message in ready_messages] == ["ready", "ready"]
    assert all(message["protocol_version"] == 1 for message in ready_messages)
    assert all(message["reconcile_required"] is True for message in ready_messages)
    assert ready_messages[0]["connection_id"] != ready_messages[1]["connection_id"]


def test_committed_event_is_broadcast_to_connected_clients(
    realtime_app: FastAPI,
    realtime_client: TestClient,
) -> None:
    with (
        realtime_client.websocket_connect(WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS) as first,
        realtime_client.websocket_connect(WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS) as second,
    ):
        first.receive_json()
        second.receive_json()
        event = ResourceEvent(
            event_type="updated",
            resource="members",
            resource_id="10",
            version=2,
        )

        publish_committed_events([event], realtime_app.state.resource_event_sink)

        first_message = first.receive_json()
        second_message = second.receive_json()

    assert first_message == second_message
    assert first_message["type"] == "resource_changed"
    assert first_message["event_type"] == "updated"
    assert first_message["resource"] == "members"
    assert first_message["resource_id"] == "10"
    assert first_message["version"] == 2
    assert first_message["occurred_at"].endswith("Z")


def test_heartbeat_ack_is_the_only_supported_client_message(
    realtime_client: TestClient,
) -> None:
    with realtime_client.websocket_connect(
        WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS
    ) as websocket:
        websocket.receive_json()
        websocket.send_json({"type": "heartbeat_ack"})
        websocket.send_json({"type": "unexpected"})

        with pytest.raises(WebSocketDisconnect) as closed:
            websocket.receive_json()

    assert closed.value.code == status.WS_1003_UNSUPPORTED_DATA


def test_server_sends_heartbeat(tmp_path: Path) -> None:
    paths = RuntimePaths.discover(tmp_path / "heartbeat runtime")
    settings = AppSettings(
        environment="test",
        realtime=RealtimeSettings(
            heartbeat_interval_seconds=0.05,
            stale_timeout_seconds=0.5,
        ),
    )
    app = create_app(
        runtime_paths=paths,
        settings=settings,
        realtime_session_verifier=lambda _token: "session-1",
    )

    with TestClient(app) as client:
        client.cookies.set(SESSION_COOKIE_NAME, "valid-token")
        with client.websocket_connect(WEBSOCKET_PATH, headers=SAME_ORIGIN_HEADERS) as websocket:
            websocket.receive_json()
            heartbeat = websocket.receive_json()
            websocket.send_json({"type": "heartbeat_ack"})

    assert heartbeat["type"] == "heartbeat"
    assert heartbeat["sent_at"].endswith("Z")
