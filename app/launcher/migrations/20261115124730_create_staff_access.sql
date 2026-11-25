-- 직원 계정, 세션, 접속 기록, 복구 코드. [ADR 0003], [ADR 0014], BR-K1~K13

-- 직원 계정. 지우지 않고 사용 중지한다 [BR-K6]
CREATE TABLE staff (
    id INTEGER PRIMARY KEY,
    -- 쓴 띄어쓰기 그대로
    name TEXT NOT NULL,
    -- [BR-K2]: 이름에서 띄어쓰기를 모두 뺀 글자. 앱이 채움
    name_key TEXT NOT NULL UNIQUE CHECK (name_key <> ''),
    -- argon2id 해시 (PHC 문자열). PIN 자체는 두지 않음
    pin_hash TEXT NOT NULL,
    -- [BR-K3]
    role TEXT NOT NULL CHECK (role IN ('admin', 'staff', 'temp_staff', 'read_only')),
    -- 쓸 수 있는 마지막 날 YYYY-MM-DD. 없으면 기한 없음
    valid_until TEXT,
    -- [BR-K6]: 1이면 사용 중지
    disabled INTEGER NOT NULL DEFAULT 0 CHECK (disabled IN (0, 1)),
    -- [BR-K7]: 연속으로 틀린 횟수, 잠금이 풀리는 시각(UTC, ISO 8601)
    failed_pin_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_pin_count >= 0),
    locked_until TEXT,
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- [BR-K5]: 임시 직원은 유효기간이 반드시 있음
    CHECK (role <> 'temp_staff' OR valid_until IS NOT NULL)
) STRICT;

-- 로그인 세션. 로그아웃하면 지움 [ADR 0014]
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    staff_id INTEGER NOT NULL REFERENCES staff (id) ON DELETE CASCADE,
    -- 증표의 SHA-256 해시 32바이트. 증표 자체는 두지 않음
    token_hash BLOB NOT NULL UNIQUE CHECK (length(token_hash) = 32),
    -- UTC, ISO 8601
    started_at TEXT NOT NULL,
    -- [BR-K8]: 여기서 10분이 지나면 잠김
    last_seen_at TEXT NOT NULL
) STRICT;

CREATE INDEX sessions_staff ON sessions (staff_id);

-- 접속 기록. 쌓기만 함 [BR-K13]
CREATE TABLE access_logs (
    id INTEGER PRIMARY KEY,
    -- UTC, ISO 8601
    occurred_at TEXT NOT NULL,
    -- 누구의 계정인지. 고른 직원이 없는 실패면 NULL
    staff_id INTEGER REFERENCES staff (id) ON DELETE NO ACTION,
    event TEXT NOT NULL CHECK (
        event IN ('login', 'login_failed', 'logout', 'unlock_failed', 'recovery_used')
    )
) STRICT;

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

-- 복구 코드. 기관에 하나뿐 [BR-K11], [BR-K12]
CREATE TABLE recovery_code (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    -- argon2id 해시 (PHC 문자열). 코드 자체는 두지 않음
    code_hash TEXT NOT NULL,
    -- UTC, ISO 8601
    created_at TEXT NOT NULL
) STRICT;
