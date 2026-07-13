"""WebSocket client와 server가 주고받는 realtime protocol schema."""

from datetime import datetime
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class RealtimeReadyMessage(BaseModel):
    type: Literal["ready"] = "ready"
    protocol_version: Literal[1] = 1
    connection_id: str
    heartbeat_interval_seconds: float
    reconcile_required: Literal[True] = True


class RealtimeHeartbeatMessage(BaseModel):
    type: Literal["heartbeat"] = "heartbeat"
    sent_at: datetime


class RealtimeResourceMessage(BaseModel):
    type: Literal["resource_changed"] = "resource_changed"
    event_type: str = Field(max_length=80)
    resource: str = Field(max_length=80)
    resource_id: str | None = Field(default=None, max_length=80)
    version: int | None = Field(default=None, ge=1)
    occurred_at: datetime


class RealtimeReconcileMessage(BaseModel):
    type: Literal["reconcile_required"] = "reconcile_required"
    reason: Literal["event_gap"] = "event_gap"


class RealtimeHeartbeatAck(BaseModel):
    model_config = ConfigDict(extra="forbid")

    type: Literal["heartbeat_ack"]
