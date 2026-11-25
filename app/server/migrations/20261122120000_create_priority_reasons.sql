-- 우선 배정 사유, 신청에 붙인 사유, 변경 기록, 추첨 때 자리 보장으로 당첨된 신청.
-- BR-S3~S5, BR-S10~S14, ADR 0017

-- 우선 배정 사유 [BR-S10]. 관리자가 만든다
CREATE TABLE priority_reasons (
    id INTEGER PRIMARY KEY,
    -- 관리자가 쓴 띄어쓰기 그대로
    name TEXT NOT NULL,
    -- 이름에서 띄어쓰기를 모두 뺀 글자. 앱이 채움
    name_key TEXT NOT NULL UNIQUE CHECK (name_key <> ''),
    -- [BR-S11]: 배정 방식. 지금은 자리 보장뿐
    method TEXT NOT NULL CHECK (method IN ('guaranteed')),
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

-- 신청에 붙인 사유 [BR-S12]. 신청 하나에 여러 개
CREATE TABLE application_priorities (
    -- 신청을 지우면(BR-D1) 함께 지운다. 지운 일은 삭제 기록에 남는다
    application_id INTEGER NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    reason_id INTEGER NOT NULL REFERENCES priority_reasons (id) ON DELETE NO ACTION,
    PRIMARY KEY (application_id, reason_id)
) STRICT;

-- 사유를 붙이고 뗀 기록 [BR-S14]. 신청이 지워질 수 있어 삭제 기록처럼 회원과 강좌도 가리킨다
CREATE TABLE priority_changes (
    id INTEGER PRIMARY KEY,
    -- 지워진 신청을 가리킬 수 있어 외래 키를 두지 않는다
    application_id INTEGER NOT NULL,
    member_id INTEGER NOT NULL REFERENCES members (id) ON DELETE NO ACTION,
    course_id INTEGER NOT NULL REFERENCES courses (id) ON DELETE NO ACTION,
    -- [BR-S10]: 한 번이라도 붙었던 사유는 이 기록 때문에 지울 수 없다
    reason_id INTEGER NOT NULL REFERENCES priority_reasons (id) ON DELETE NO ACTION,
    change TEXT NOT NULL CHECK (change IN ('added', 'removed')),
    changed_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    changed_at TEXT NOT NULL
) STRICT;

-- 사유·회원·강좌가 쓰이는지 볼 때
CREATE INDEX priority_changes_reason ON priority_changes (reason_id);
CREATE INDEX priority_changes_member ON priority_changes (member_id);
CREATE INDEX priority_changes_course ON priority_changes (course_id);

CREATE TRIGGER priority_changes_no_update BEFORE UPDATE ON priority_changes
BEGIN
    SELECT RAISE(ABORT, 'priority_changes는 고칠 수 없음');
END;

CREATE TRIGGER priority_changes_no_delete BEFORE DELETE ON priority_changes
BEGIN
    SELECT RAISE(ABORT, 'priority_changes는 지울 수 없음');
END;

-- 추첨 때 자리 보장으로 당첨된 신청 [BR-S4]. 추첨 대상(lottery_entries)과 겹치지 않는다
CREATE TABLE lottery_guaranteed_entries (
    lottery_run_id INTEGER NOT NULL REFERENCES lottery_runs (id) ON DELETE NO ACTION,
    -- 추첨 기록에 남은 신청은 지울 수 없다
    application_id INTEGER NOT NULL REFERENCES applications (id) ON DELETE NO ACTION,
    PRIMARY KEY (lottery_run_id, application_id)
) STRICT;

CREATE INDEX lottery_guaranteed_entries_application
    ON lottery_guaranteed_entries (application_id);

CREATE TRIGGER lottery_guaranteed_entries_no_update BEFORE UPDATE ON lottery_guaranteed_entries
BEGIN
    SELECT RAISE(ABORT, 'lottery_guaranteed_entries는 고칠 수 없음');
END;

CREATE TRIGGER lottery_guaranteed_entries_no_delete BEFORE DELETE ON lottery_guaranteed_entries
BEGIN
    SELECT RAISE(ABORT, 'lottery_guaranteed_entries는 지울 수 없음');
END;
