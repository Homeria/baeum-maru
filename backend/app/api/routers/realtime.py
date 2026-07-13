"""다중 사용자 갱신 알림을 전달하는 인증 WebSocket endpoint."""

from typing import Annotated

from fastapi import APIRouter, Depends, WebSocket, status
from pydantic import ValidationError
from starlette.websockets import WebSocketDisconnect

from app.api.dependencies import require_realtime_session
from app.api.realtime import RealtimeHub
from app.schemas.realtime import RealtimeHeartbeatAck

router = APIRouter(tags=["realtime"])


@router.websocket("/events/ws")
async def realtime_events(
    websocket: WebSocket,
    _session_id: Annotated[str, Depends(require_realtime_session)],
) -> None:
    hub: RealtimeHub = websocket.app.state.realtime_hub
    connection_id = await hub.connect(websocket)
    if connection_id is None:
        return

    try:
        while True:
            payload = await websocket.receive_json()
            RealtimeHeartbeatAck.model_validate(payload)
            hub.mark_seen(connection_id)
    except WebSocketDisconnect:
        pass
    except (ValidationError, ValueError):
        await hub.close_connection(
            connection_id,
            code=status.WS_1003_UNSUPPORTED_DATA,
            reason="Unsupported realtime message",
        )
    finally:
        hub.disconnect(connection_id)
