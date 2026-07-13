"""업무 변경과 감사 로그가 같은 transaction에 남는지 검증한다."""

import pytest
from sqlalchemy import func, select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import Session, sessionmaker

from app.models.members import Member
from app.models.operations import AuditLog
from app.services.operation_service import record_audit
from app.services.realtime_service import ResourceEvent, publish_committed_events


def test_business_change_and_audit_commit_together(
    migrated_session_factory: sessionmaker[Session],
) -> None:
    observed_counts: list[tuple[int, int]] = []

    def observe_after_commit(_event: ResourceEvent) -> None:
        with migrated_session_factory() as observer:
            member_count = observer.scalar(select(func.count()).select_from(Member)) or 0
            audit_count = observer.scalar(select(func.count()).select_from(AuditLog)) or 0
            observed_counts.append((member_count, audit_count))

    with migrated_session_factory() as session:
        member = Member(member_no="10-12345", name="남복심", phone="010-0000-0000")
        session.add(member)
        session.flush()
        record_audit(
            session,
            actor_kind="system",
            action="member.created",
            resource_type="members",
            resource_id=str(member.id),
            summary="회원 등록",
            request_id="request-1",
            metadata={"member_id": member.id},
        )
        event = ResourceEvent(
            event_type="created",
            resource="members",
            resource_id=str(member.id),
            version=member.version,
        )

        session.commit()
        publish_committed_events([event], observe_after_commit)

    assert observed_counts == [(1, 1)]


def test_failed_transaction_rolls_back_audit_and_does_not_publish(
    migrated_session_factory: sessionmaker[Session],
) -> None:
    with migrated_session_factory() as session:
        session.add(Member(member_no="10-12345", name="기존 회원", phone="010-0000-0000"))
        session.commit()

    published: list[ResourceEvent] = []
    with migrated_session_factory() as session:
        event = ResourceEvent(event_type="created", resource="members", resource_id="2")
        try:
            record_audit(
                session,
                actor_kind="system",
                action="member.created",
                resource_type="members",
                resource_id="2",
                summary="중복 회원 등록 시도",
            )
            session.add(Member(member_no="10-12345", name="중복 회원", phone="010-1111-1111"))
            session.commit()
        except IntegrityError:
            session.rollback()
        else:
            publish_committed_events([event], published.append)

    with migrated_session_factory() as session:
        assert session.scalar(select(func.count()).select_from(Member)) == 1
        assert session.scalar(select(func.count()).select_from(AuditLog)) == 0
    assert published == []


def test_audit_rejects_sensitive_or_invalid_metadata(
    migrated_session_factory: sessionmaker[Session],
) -> None:
    with migrated_session_factory() as session, pytest.raises(ValueError, match="not allowed"):
        record_audit(
            session,
            actor_kind="system",
            action="member.updated",
            resource_type="members",
            summary="회원 수정",
            metadata={"changes": {"phone": "010-1234-5678"}},
        )


def test_user_actor_requires_user_id(migrated_session_factory: sessionmaker[Session]) -> None:
    with migrated_session_factory() as session, pytest.raises(ValueError, match="actor_user_id"):
        record_audit(
            session,
            actor_kind="user",
            action="member.updated",
            resource_type="members",
            summary="회원 수정",
        )
