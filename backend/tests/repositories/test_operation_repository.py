"""감사 repository가 현재 transaction만 변경하는지 검증한다."""

from sqlalchemy import func, select
from sqlalchemy.orm import Session, sessionmaker

from app.models.operations import AuditLog
from app.repositories.operation_repository import add_audit_log


def test_add_audit_log_flushes_without_committing(
    migrated_session_factory: sessionmaker[Session],
) -> None:
    with migrated_session_factory() as session:
        audit_log = add_audit_log(
            session,
            actor_kind="system",
            actor_user_id=None,
            actor_access_code_id=None,
            actor_display_name=None,
            action="member.created",
            resource_type="members",
            resource_id="1",
            summary="회원 등록",
            request_id="request-1",
            metadata_json={"member_id": 1},
        )

        assert audit_log.id is not None
        session.rollback()

    with migrated_session_factory() as session:
        assert session.scalar(select(func.count()).select_from(AuditLog)) == 0
