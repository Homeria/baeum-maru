"""감사 로그 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS audit_logs (
        id INTEGER PRIMARY KEY,
        actor_kind TEXT NOT NULL CHECK (actor_kind IN ('operator', 'launcher', 'system')),
        actor_operator_id INTEGER REFERENCES operators(id) ON DELETE RESTRICT,
        actor_access_code_id INTEGER REFERENCES access_codes(id) ON DELETE RESTRICT,
        actor_display_name TEXT,
        action TEXT NOT NULL,
        resource_type TEXT NOT NULL,
        resource_id TEXT,
        summary TEXT NOT NULL,
        request_id TEXT,
        metadata_json TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    "CREATE INDEX IF NOT EXISTS ix_audit_logs_created_at ON audit_logs(created_at)",
    """
    CREATE INDEX IF NOT EXISTS ix_audit_logs_resource
    ON audit_logs(resource_type, resource_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_audit_logs_actor_operator_id
    ON audit_logs(actor_operator_id)
    """,
)
