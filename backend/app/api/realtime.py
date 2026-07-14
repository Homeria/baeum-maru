"""WebSocket 연결, heartbeat와 resource event broadcast 상태를 관리한다."""

import asyncio
import logging
from contextlib import suppress
from dataclasses import dataclass, field
from time import monotonic
from uuid import uuid4

from fastapi import WebSocket, status

from app.core.settings import RealtimeSettings
from app.core.time import utc_now
from app.schemas.realtime import (
    RealtimeHeartbeatMessage,
    RealtimeReadyMessage,
    RealtimeReconcileMessage,
    RealtimeResourceMessage,
)
from app.services.realtime_service import ResourceEvent

logger = logging.getLogger(__name__)


@dataclass(slots=True)
class _Connection:
    websocket: WebSocket
    last_seen: float = field(default_factory=monotonic)
    send_lock: asyncio.Lock = field(default_factory=asyncio.Lock)


class RealtimeHub:
    """한 server process 안의 소수 WebSocket 연결에 변경 알림을 전달한다."""

    def __init__(self, settings: RealtimeSettings) -> None:
        self.settings = settings
        self._connections: dict[str, _Connection] = {}
        self._loop: asyncio.AbstractEventLoop | None = None
        self._queue: asyncio.Queue[ResourceEvent] | None = None
        self._tasks: list[asyncio.Task[None]] = []
        self._reconcile_pending = False

    @property
    def connected_count(self) -> int:
        return len(self._connections)

    async def start(self) -> None:
        self._loop = asyncio.get_running_loop()
        self._queue = asyncio.Queue(maxsize=self.settings.event_queue_size)
        self._tasks = [
            asyncio.create_task(self._broadcast_loop(), name="realtime-broadcast"),
            asyncio.create_task(self._heartbeat_loop(), name="realtime-heartbeat"),
        ]

    async def stop(self) -> None:
        tasks, self._tasks = self._tasks, []
        for task in tasks:
            task.cancel()
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)

        connections, self._connections = self._connections, {}
        if connections:
            await asyncio.gather(
                *(
                    self._close_socket(
                        connection.websocket,
                        code=status.WS_1012_SERVICE_RESTART,
                        reason="Server shutting down",
                    )
                    for connection in connections.values()
                )
            )
        self._queue = None
        self._loop = None
        self._reconcile_pending = False

    async def connect(self, websocket: WebSocket) -> str | None:
        await websocket.accept()
        if len(self._connections) >= self.settings.max_connections:
            await self._close_socket(
                websocket,
                code=status.WS_1013_TRY_AGAIN_LATER,
                reason="Too many connections",
            )
            return None

        connection_id = str(uuid4())
        connection = _Connection(websocket=websocket)
        self._connections[connection_id] = connection
        ready = RealtimeReadyMessage(
            connection_id=connection_id,
            heartbeat_interval_seconds=self.settings.heartbeat_interval_seconds,
        )
        if not await self._send(connection, ready.model_dump(mode="json")):
            await self.close_connection(
                connection_id,
                code=status.WS_1011_INTERNAL_ERROR,
                reason="Realtime delivery failed",
            )
            return None
        return connection_id

    def disconnect(self, connection_id: str) -> None:
        self._connections.pop(connection_id, None)

    async def close_connection(self, connection_id: str, *, code: int, reason: str) -> None:
        connection = self._connections.pop(connection_id, None)
        if connection is not None:
            await self._close_socket(connection.websocket, code=code, reason=reason)

    def mark_seen(self, connection_id: str) -> None:
        connection = self._connections.get(connection_id)
        if connection is not None:
            connection.last_seen = monotonic()

    def publish(self, event: ResourceEvent) -> None:
        """sync service thread에서도 호출할 수 있도록 event loop에 enqueue를 예약한다."""
        loop = self._loop
        if loop is None or loop.is_closed():
            logger.warning("Realtime event dropped because hub is not running")
            return
        try:
            loop.call_soon_threadsafe(self._enqueue, event)
        except RuntimeError:
            logger.warning("Realtime event dropped because event loop is closed")

    def _enqueue(self, event: ResourceEvent) -> None:
        queue = self._queue
        if queue is None:
            return
        try:
            queue.put_nowait(event)
        except asyncio.QueueFull:
            self._reconcile_pending = True
            logger.error(
                "Realtime event queue is full",
                extra={"resource": event.resource, "resource_id": event.resource_id},
            )

    async def _broadcast_loop(self) -> None:
        queue = self._queue
        if queue is None:
            return
        while True:
            event = await queue.get()
            try:
                message = RealtimeResourceMessage(
                    event_type=event.event_type,
                    resource=event.resource,
                    resource_id=event.resource_id,
                    version=event.version,
                    occurred_at=event.occurred_at,
                )
                await self._broadcast(message.model_dump(mode="json"))
                if self._reconcile_pending:
                    self._reconcile_pending = False
                    reconcile = RealtimeReconcileMessage()
                    await self._broadcast(reconcile.model_dump(mode="json"))
            finally:
                queue.task_done()

    async def _heartbeat_loop(self) -> None:
        while True:
            await asyncio.sleep(self.settings.heartbeat_interval_seconds)
            stale_before = monotonic() - self.settings.stale_timeout_seconds
            stale_ids = [
                connection_id
                for connection_id, connection in self._connections.items()
                if connection.last_seen < stale_before
            ]
            for connection_id in stale_ids:
                await self.close_connection(
                    connection_id,
                    code=status.WS_1001_GOING_AWAY,
                    reason="Heartbeat timeout",
                )

            heartbeat = RealtimeHeartbeatMessage(sent_at=utc_now())
            await self._broadcast(heartbeat.model_dump(mode="json"))

    async def _broadcast(self, payload: dict[str, object]) -> None:
        connection_items = list(self._connections.items())
        if not connection_items:
            return
        results = await asyncio.gather(
            *(self._send(connection, payload) for _, connection in connection_items)
        )
        failed_ids = [
            connection_id
            for (connection_id, _), sent in zip(connection_items, results, strict=True)
            if not sent
        ]
        if failed_ids:
            await asyncio.gather(
                *(
                    self.close_connection(
                        connection_id,
                        code=status.WS_1011_INTERNAL_ERROR,
                        reason="Realtime delivery failed",
                    )
                    for connection_id in failed_ids
                )
            )

    async def _send(self, connection: _Connection, payload: dict[str, object]) -> bool:
        try:
            async with connection.send_lock:
                await asyncio.wait_for(
                    connection.websocket.send_json(payload),
                    timeout=self.settings.send_timeout_seconds,
                )
            return True
        except Exception:
            return False

    @staticmethod
    async def _close_socket(websocket: WebSocket, *, code: int, reason: str) -> None:
        with suppress(Exception):
            await websocket.close(code=code, reason=reason)
