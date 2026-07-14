"""감사 repository가 현재 sqlite3 transaction만 변경하는지 검증한다."""

from app.db.database import Database
from app.repositories.operation_repository import add_audit_log


def test_add_audit_log_inserts_without_committing(initialized_database: Database) -> None:
    with initialized_database.connection() as connection:
        audit_log = add_audit_log(
            connection,
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
        assert audit_log.id > 0
        assert audit_log.metadata_json == {"member_id": 1}
        connection.rollback()

    with initialized_database.connection() as connection:
        count = connection.execute("SELECT COUNT(*) FROM audit_logs").fetchone()[0]
    assert count == 0
