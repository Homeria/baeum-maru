"""감사 로그와 백그라운드 작업 보조 테이블 DDL."""

STATEMENTS = (
    """
    CREATE TABLE IF NOT EXISTS audit_logs (
        id INTEGER PRIMARY KEY,
        actor_kind TEXT NOT NULL CHECK (actor_kind IN ('user', 'launcher', 'system')),
        actor_user_id INTEGER REFERENCES users(id) ON DELETE RESTRICT,
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
    """
    CREATE TABLE IF NOT EXISTS operation_jobs (
        id INTEGER PRIMARY KEY,
        job_type TEXT NOT NULL
            CHECK (job_type IN ('import', 'export', 'backup', 'restore', 'notification')),
        status TEXT NOT NULL DEFAULT 'queued'
            CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled')),
        source_name TEXT,
        output_relative_path TEXT,
        requested_by_user_id INTEGER REFERENCES users(id) ON DELETE RESTRICT,
        requested_by_access_code_id INTEGER REFERENCES access_codes(id) ON DELETE RESTRICT,
        total_count INTEGER NOT NULL DEFAULT 0,
        success_count INTEGER NOT NULL DEFAULT 0,
        failure_count INTEGER NOT NULL DEFAULT 0,
        error_summary TEXT,
        metadata_json TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        started_at TEXT,
        completed_at TEXT,
        CHECK (total_count >= 0 AND success_count >= 0 AND failure_count >= 0),
        CHECK (success_count + failure_count <= total_count)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS operation_job_errors (
        id INTEGER PRIMARY KEY,
        job_id INTEGER NOT NULL REFERENCES operation_jobs(id) ON DELETE CASCADE,
        row_number INTEGER CHECK (row_number IS NULL OR row_number >= 1),
        field_name TEXT,
        message TEXT NOT NULL,
        raw_value TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS idempotency_records (
        id INTEGER PRIMARY KEY,
        namespace TEXT NOT NULL,
        key_hash TEXT NOT NULL,
        request_hash TEXT NOT NULL,
        status TEXT NOT NULL CHECK (status IN ('processing', 'completed', 'failed')),
        response_status INTEGER,
        response_json TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT NOT NULL CHECK (expires_at > created_at),
        UNIQUE (namespace, key_hash)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS operation_locks (
        resource_type TEXT NOT NULL,
        resource_id TEXT NOT NULL,
        operation TEXT NOT NULL,
        owner_token TEXT NOT NULL,
        acquired_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT NOT NULL CHECK (expires_at > acquired_at),
        PRIMARY KEY (resource_type, resource_id)
    )
    """,
    "CREATE INDEX IF NOT EXISTS ix_audit_logs_created_at ON audit_logs(created_at)",
    """
    CREATE INDEX IF NOT EXISTS ix_audit_logs_resource
    ON audit_logs(resource_type, resource_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_audit_logs_actor_user_id
    ON audit_logs(actor_user_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_operation_jobs_type_status
    ON operation_jobs(job_type, status)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_operation_job_errors_job_id
    ON operation_job_errors(job_id)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_idempotency_records_expires_at
    ON idempotency_records(expires_at)
    """,
    """
    CREATE INDEX IF NOT EXISTS ix_operation_locks_expires_at
    ON operation_locks(expires_at)
    """,
)
