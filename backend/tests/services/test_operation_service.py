"""업무 변경과 감사 로그가 같은 sqlite3 transaction에 남는지 검증한다."""

import sqlite3

import pytest

from app.db.database import Database
from app.services.operation_service import record_audit
from app.services.realtime_service import ResourceEvent, publish_committed_events


def _count(database: Database, table_name: str) -> int:
    with database.connection() as connection:
        return int(connection.execute(f'SELECT COUNT(*) FROM "{table_name}"').fetchone()[0])


def test_business_change_and_audit_commit_together(initialized_database: Database) -> None:
    observed_counts: list[tuple[int, int]] = []

    def observe_after_commit(_event: ResourceEvent) -> None:
        observed_counts.append(
            (_count(initialized_database, "members"), _count(initialized_database, "audit_logs"))
        )

    with initialized_database.transaction() as connection:
        cursor = connection.execute(
            "INSERT INTO members (member_no, name, phone) VALUES (?, ?, ?)",
            ("10-12345", "남복심", "010-0000-0000"),
        )
        member_id = cursor.lastrowid
        assert member_id is not None
        record_audit(
            connection,
            actor_kind="system",
            action="member.created",
            resource_type="members",
            resource_id=str(member_id),
            summary="회원 등록",
            request_id="request-1",
            metadata={"member_id": member_id},
        )
        event = ResourceEvent(
            event_type="created",
            resource="members",
            resource_id=str(member_id),
            version=1,
        )

    publish_committed_events([event], observe_after_commit)
    assert observed_counts == [(1, 1)]


def test_failed_transaction_rolls_back_audit_and_does_not_publish(
    initialized_database: Database,
) -> None:
    with initialized_database.transaction() as connection:
        connection.execute(
            "INSERT INTO members (member_no, name, phone) VALUES (?, ?, ?)",
            ("10-12345", "기존 회원", "010-0000-0000"),
        )

    published: list[ResourceEvent] = []
    event = ResourceEvent(event_type="created", resource="members", resource_id="2")
    try:
        with initialized_database.transaction() as connection:
            record_audit(
                connection,
                actor_kind="system",
                action="member.created",
                resource_type="members",
                resource_id="2",
                summary="중복 회원 등록 시도",
            )
            connection.execute(
                "INSERT INTO members (member_no, name, phone) VALUES (?, ?, ?)",
                ("10-12345", "중복 회원", "010-1111-1111"),
            )
    except sqlite3.IntegrityError:
        pass
    else:
        publish_committed_events([event], published.append)

    assert _count(initialized_database, "members") == 1
    assert _count(initialized_database, "audit_logs") == 0
    assert published == []


def test_audit_rejects_sensitive_or_invalid_metadata(initialized_database: Database) -> None:
    with (
        initialized_database.connection() as connection,
        pytest.raises(ValueError, match="not allowed"),
    ):
        record_audit(
            connection,
            actor_kind="system",
            action="member.updated",
            resource_type="members",
            summary="회원 수정",
            metadata={"changes": {"phone": "010-1234-5678"}},
        )


def test_user_actor_requires_user_id(initialized_database: Database) -> None:
    with (
        initialized_database.connection() as connection,
        pytest.raises(ValueError, match="actor_user_id"),
    ):
        record_audit(
            connection,
            actor_kind="user",
            action="member.updated",
            resource_type="members",
            summary="회원 수정",
        )
