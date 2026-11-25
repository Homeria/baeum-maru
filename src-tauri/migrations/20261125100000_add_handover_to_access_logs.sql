-- [BR-K13]: 접속 기록에 호스트 넘기기, 넘기기 취소, 넘겨받기를 남긴다 (BR-H1~H3)
-- SQLite는 CHECK를 고칠 수 없어서 같은 모양의 새 테이블로 옮긴다. 기록은 그대로 옮기고 id도 지킨다.
-- 넘겨받기('take_over')도 함께 넣어 테이블을 한 번만 옮긴다.

DROP TRIGGER access_logs_no_update;
DROP TRIGGER access_logs_no_delete;

CREATE TABLE access_logs_new (
    id INTEGER PRIMARY KEY,
    -- UTC, ISO 8601
    occurred_at TEXT NOT NULL,
    -- 누구의 계정인지. 고른 직원이 없는 실패나, 복원한 관리자가 그 백업에 없으면 NULL
    staff_id INTEGER REFERENCES staff (id) ON DELETE NO ACTION,
    -- 어느 기기의 일인지. 호스트 PC 자신이면 NULL
    device_id INTEGER REFERENCES devices (id) ON DELETE NO ACTION,
    event TEXT NOT NULL CHECK (
        event IN (
            'login', 'login_failed', 'logout', 'unlock_failed', 'recovery_used', 'restore',
            'device_approved', 'device_rejected', 'device_revoked', 'device_approval_failed',
            'run_mode_changed', 'handover', 'handover_canceled', 'take_over'
        )
    )
) STRICT;

INSERT INTO access_logs_new (id, occurred_at, staff_id, device_id, event)
SELECT id, occurred_at, staff_id, device_id, event FROM access_logs;

DROP TABLE access_logs;
ALTER TABLE access_logs_new RENAME TO access_logs;

CREATE INDEX access_logs_occurred ON access_logs (occurred_at);

-- [BR-K13]: 접속 기록은 고치거나 지우지 않음
CREATE TRIGGER access_logs_no_update BEFORE UPDATE ON access_logs
BEGIN
    SELECT RAISE(ABORT, 'access_logs는 고칠 수 없음');
END;

CREATE TRIGGER access_logs_no_delete BEFORE DELETE ON access_logs
BEGIN
    SELECT RAISE(ABORT, 'access_logs는 지울 수 없음');
END;
