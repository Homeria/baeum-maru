"""transaction 완료 후 resource 변경 event를 연결된 사용자에게 알리는 service."""

import logging
from collections.abc import Callable, Iterable
from dataclasses import dataclass, field
from datetime import datetime

from app.core.time import utc_now

logger = logging.getLogger(__name__)


@dataclass(frozen=True, slots=True)
class ResourceEvent:
    """REST 응답을 다시 조회하도록 알리는 개인정보 없는 변경 신호."""

    event_type: str
    resource: str
    resource_id: str | None = None
    version: int | None = None
    occurred_at: datetime = field(default_factory=utc_now)

    def __post_init__(self) -> None:
        if not self.event_type.strip() or len(self.event_type) > 80:
            raise ValueError("event_type must contain at most 80 characters")
        if not self.resource.strip() or len(self.resource) > 80:
            raise ValueError("resource must contain at most 80 characters")
        if self.resource_id is not None and (
            not self.resource_id.strip() or len(self.resource_id) > 80
        ):
            raise ValueError("resource_id must contain at most 80 characters")
        if self.version is not None and self.version < 1:
            raise ValueError("version must be positive")
        if self.occurred_at.utcoffset() is None:
            raise ValueError("occurred_at must include timezone information")


type ResourceEventSink = Callable[[ResourceEvent], None]


def publish_committed_events(
    events: Iterable[ResourceEvent],
    sink: ResourceEventSink | None,
) -> None:
    """commit 뒤 이벤트를 순서대로 전달한다. 전달 실패는 이미 끝난 업무를 되돌리지 않는다."""
    if sink is None:
        return

    for event in events:
        try:
            sink(event)
        except Exception:
            logger.exception(
                "Failed to publish committed resource event",
                extra={
                    "event_type": event.event_type,
                    "resource": event.resource,
                    "resource_id": event.resource_id,
                },
            )
