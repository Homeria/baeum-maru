"""RealtimeHub의 queue overflow와 stale connection 정리를 검증한다."""

import asyncio
from typing import Any, cast

import pytest
from fastapi import WebSocket
from starlette import status

from app.api.realtime import RealtimeHub
from app.core.settings import RealtimeSettings
from app.services.realtime_service import ResourceEvent


class FakeWebSocket:
    def __init__(self) -> None:
        self.messages: list[dict[str, Any]] = []
        self.send_gate = asyncio.Event()
        self.send_gate.set()
        self.closed = asyncio.Event()
        self.close_code: int | None = None

    async def accept(self) -> None:
        pass

    async def send_json(self, payload: dict[str, Any]) -> None:
        await self.send_gate.wait()
        self.messages.append(payload)

    async def close(self, code: int = 1000, reason: str | None = None) -> None:
        self.close_code = code
        self.closed.set()


async def wait_for_message(fake: FakeWebSocket, message_type: str) -> None:
    async def message_arrived() -> None:
        while not any(message.get("type") == message_type for message in fake.messages):
            await asyncio.sleep(0.005)

    await asyncio.wait_for(message_arrived(), timeout=1)


@pytest.mark.asyncio
async def test_queue_overflow_requests_full_reconciliation() -> None:
    hub = RealtimeHub(
        RealtimeSettings(
            heartbeat_interval_seconds=10,
            stale_timeout_seconds=20,
            event_queue_size=16,
        )
    )
    fake = FakeWebSocket()
    await hub.start()
    try:
        await hub.connect(cast(WebSocket, fake))
        fake.send_gate.clear()
        hub.publish(ResourceEvent(event_type="updated", resource="members", resource_id="0"))
        await asyncio.sleep(0.01)

        for item_id in range(1, 25):
            hub.publish(
                ResourceEvent(
                    event_type="updated",
                    resource="members",
                    resource_id=str(item_id),
                )
            )
        await asyncio.sleep(0.01)
        fake.send_gate.set()

        await wait_for_message(fake, "reconcile_required")
    finally:
        await hub.stop()

    assert any(
        message == {"type": "reconcile_required", "reason": "event_gap"}
        for message in fake.messages
    )


@pytest.mark.asyncio
async def test_stale_connection_is_closed_after_heartbeat_timeout() -> None:
    hub = RealtimeHub(
        RealtimeSettings(
            heartbeat_interval_seconds=0.05,
            stale_timeout_seconds=0.11,
        )
    )
    fake = FakeWebSocket()
    await hub.start()
    try:
        await hub.connect(cast(WebSocket, fake))
        await asyncio.wait_for(fake.closed.wait(), timeout=0.5)
    finally:
        await hub.stop()

    assert fake.close_code == status.WS_1001_GOING_AWAY
    assert hub.connected_count == 0


@pytest.mark.asyncio
async def test_slow_connection_is_closed_after_send_timeout() -> None:
    hub = RealtimeHub(
        RealtimeSettings(
            heartbeat_interval_seconds=10,
            stale_timeout_seconds=20,
            send_timeout_seconds=0.1,
        )
    )
    fake = FakeWebSocket()
    await hub.start()
    try:
        await hub.connect(cast(WebSocket, fake))
        fake.send_gate.clear()
        hub.publish(ResourceEvent(event_type="updated", resource="members", resource_id="1"))
        await asyncio.wait_for(fake.closed.wait(), timeout=0.5)
    finally:
        await hub.stop()

    assert fake.close_code == status.WS_1011_INTERNAL_ERROR
    assert hub.connected_count == 0
