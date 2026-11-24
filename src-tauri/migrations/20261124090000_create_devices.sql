-- 기기 승인 [ADR 0003], [ADR 0007], BR-K13~K16

-- 승인한 기기. 승인을 취소해도 지우지 않는다 (접속 기록이 가리킨다)
CREATE TABLE devices (
    id INTEGER PRIMARY KEY,
    -- 요청한 PC가 보낸 이름 (예: 컴퓨터 이름)
    name TEXT NOT NULL CHECK (name <> ''),
    -- 기기 증표의 SHA-256 해시 32바이트. 증표 자체는 두지 않음
    token_hash BLOB NOT NULL UNIQUE CHECK (length(token_hash) = 32),
    -- UTC, ISO 8601
    approved_at TEXT NOT NULL,
    approved_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- [BR-K16]: 승인을 취소한 시각. 있으면 이 증표는 쓸 수 없음
    revoked_at TEXT
) STRICT;

-- 승인 요청. [BR-K15]: 5분 동안만 허락할 수 있음
CREATE TABLE device_requests (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL CHECK (name <> ''),
    -- 승인 번호 6자리. 요청한 PC와 호스트 화면에 함께 뜸
    code TEXT NOT NULL CHECK (length(code) = 6),
    -- 요청한 PC가 받은 기기 증표의 해시. 허락하면 그대로 devices로 옮김
    token_hash BLOB NOT NULL UNIQUE CHECK (length(token_hash) = 32),
    -- UTC, ISO 8601
    requested_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    -- 허락했으면 만든 기기
    device_id INTEGER REFERENCES devices (id) ON DELETE NO ACTION
) STRICT;

CREATE INDEX device_requests_pending ON device_requests (status, requested_at);

-- [BR-K13]: 접속 기록에 기기와 기기 승인, 실행 모드 바꾸기를 남긴다
-- SQLite는 CHECK를 고칠 수 없어서 같은 모양의 새 테이블로 옮긴다. 기록은 그대로 옮기고 id도 지킨다

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
            'run_mode_changed'
        )
    )
) STRICT;

INSERT INTO access_logs_new (id, occurred_at, staff_id, event)
SELECT id, occurred_at, staff_id, event FROM access_logs;

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
