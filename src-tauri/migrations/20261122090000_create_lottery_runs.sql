-- 추첨 기록과 선발 마감. BR-L1~L6, ADR 0016
-- 기록은 고치거나 지우지 않는다. 재추첨 때 이전 기록에 취소 표시만 한다(BR-L4).

-- 추첨 기록 [BR-L1]. 전원 당첨(BR-S6)도 남긴다
CREATE TABLE lottery_runs (
    id INTEGER PRIMARY KEY,
    course_id INTEGER NOT NULL REFERENCES courses (id) ON DELETE NO ACTION,
    run_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    run_at TEXT NOT NULL,
    -- 16자리 16진수(소문자)
    seed TEXT NOT NULL CHECK (length(seed) = 16 AND seed NOT GLOB '*[^0-9a-f]*'),
    -- 추첨 방식 버전(ADR 0016)
    version INTEGER NOT NULL CHECK (version >= 1),
    -- 추첨한 때의 정원. 강좌의 정원이 나중에 바뀌어도 결과를 다시 만들 수 있게(BR-L2)
    capacity INTEGER NOT NULL CHECK (capacity >= 1),
    -- [BR-L4]: 재추첨하면 채운다. 둘 다 있거나 둘 다 없다
    cancelled_by INTEGER REFERENCES staff (id) ON DELETE NO ACTION,
    cancelled_at TEXT,
    CHECK ((cancelled_by IS NULL) = (cancelled_at IS NULL))
) STRICT;

-- 강좌마다 취소되지 않은 추첨은 하나
CREATE UNIQUE INDEX lottery_runs_one_active
    ON lottery_runs (course_id)
    WHERE cancelled_at IS NULL;

-- 취소 표시를 한 번 채우는 것만 된다
CREATE TRIGGER lottery_runs_only_cancel BEFORE UPDATE ON lottery_runs
WHEN OLD.cancelled_at IS NOT NULL
    OR NEW.id IS NOT OLD.id
    OR NEW.course_id IS NOT OLD.course_id
    OR NEW.run_by IS NOT OLD.run_by
    OR NEW.run_at IS NOT OLD.run_at
    OR NEW.seed IS NOT OLD.seed
    OR NEW.version IS NOT OLD.version
    OR NEW.capacity IS NOT OLD.capacity
BEGIN
    SELECT RAISE(ABORT, 'lottery_runs는 취소 표시만 한 번 할 수 있음');
END;

CREATE TRIGGER lottery_runs_no_delete BEFORE DELETE ON lottery_runs
BEGIN
    SELECT RAISE(ABORT, 'lottery_runs는 지울 수 없음');
END;

-- 추첨 대상 신청과 뽑힌 순서. 1부터 정원까지가 당첨, 그 뒤는 차례로 대기 순번 1, 2, …
CREATE TABLE lottery_entries (
    lottery_run_id INTEGER NOT NULL REFERENCES lottery_runs (id) ON DELETE NO ACTION,
    -- 추첨 대상이었던 신청은 지울 수 없다
    application_id INTEGER NOT NULL REFERENCES applications (id) ON DELETE NO ACTION,
    position INTEGER NOT NULL CHECK (position >= 1),
    PRIMARY KEY (lottery_run_id, application_id),
    UNIQUE (lottery_run_id, position)
) STRICT;

-- 신청이 추첨 대상이었는지 볼 때
CREATE INDEX lottery_entries_application ON lottery_entries (application_id);

CREATE TRIGGER lottery_entries_no_update BEFORE UPDATE ON lottery_entries
BEGIN
    SELECT RAISE(ABORT, 'lottery_entries는 고칠 수 없음');
END;

CREATE TRIGGER lottery_entries_no_delete BEFORE DELETE ON lottery_entries
BEGIN
    SELECT RAISE(ABORT, 'lottery_entries는 지울 수 없음');
END;

-- 선발 마감 [BR-L4, BR-L5]. 강좌마다 한 번
CREATE TABLE selection_closures (
    course_id INTEGER PRIMARY KEY REFERENCES courses (id) ON DELETE NO ACTION,
    closed_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    closed_at TEXT NOT NULL
) STRICT;

CREATE TRIGGER selection_closures_no_update BEFORE UPDATE ON selection_closures
BEGIN
    SELECT RAISE(ABORT, 'selection_closures는 고칠 수 없음');
END;

CREATE TRIGGER selection_closures_no_delete BEFORE DELETE ON selection_closures
BEGIN
    SELECT RAISE(ABORT, 'selection_closures는 지울 수 없음');
END;
