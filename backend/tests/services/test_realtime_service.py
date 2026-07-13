"""commit 이후 resource event 전달의 안정성을 검증한다."""

import logging
from datetime import UTC

import pytest

from app.services.realtime_service import ResourceEvent, publish_committed_events


def test_resource_event_uses_timezone_aware_timestamp() -> None:
    event = ResourceEvent(event_type="updated", resource="members", resource_id="1", version=2)

    assert event.occurred_at.tzinfo is UTC


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("event_type", ""),
        ("resource", " "),
        ("resource_id", ""),
        ("version", 0),
    ],
)
def test_resource_event_rejects_invalid_contract(field: str, value: str | int) -> None:
    values: dict[str, object] = {"event_type": "updated", "resource": "members"}
    values[field] = value

    with pytest.raises(ValueError):
        ResourceEvent(**values)  # type: ignore[arg-type]


def test_publish_continues_when_one_sink_delivery_fails(caplog: pytest.LogCaptureFixture) -> None:
    first = ResourceEvent(event_type="updated", resource="members", resource_id="1")
    second = ResourceEvent(event_type="updated", resource="members", resource_id="2")
    attempted: list[str | None] = []

    def sink(event: ResourceEvent) -> None:
        attempted.append(event.resource_id)
        if event.resource_id == "1":
            raise RuntimeError("disconnected client")

    with caplog.at_level(logging.ERROR):
        publish_committed_events([first, second], sink)

    assert attempted == ["1", "2"]
    assert "Failed to publish committed resource event" in caplog.text
