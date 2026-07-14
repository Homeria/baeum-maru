"""애플리케이션이 공유하는 UTC 시각 helper."""

from datetime import UTC, datetime


def utc_now() -> datetime:
    return datetime.now(UTC)
