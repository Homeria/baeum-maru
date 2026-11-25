-- 신청, 규칙 예외, 삭제 기록. BR-A1~A7, BR-D1~D3
-- 회원·강좌·직원 지우기는 courses와 같은 까닭으로 NO ACTION으로 막는다.

-- 신청. 회원 한 명 x 강좌 하나
CREATE TABLE applications (
    id INTEGER PRIMARY KEY,
    -- [BR-M5]: 신청이 있는 회원은 지울 수 없음
    member_id INTEGER NOT NULL REFERENCES members (id) ON DELETE NO ACTION,
    -- [BR-T6]: 신청이 있는 강좌는 지울 수 없음
    course_id INTEGER NOT NULL REFERENCES courses (id) ON DELETE NO ACTION,
    -- 용어집의 신청 상태. 지금은 submitted(접수됨)만 쓰고 나머지는 선발에서 쓴다
    status TEXT NOT NULL CHECK (
        status IN ('submitted', 'selected', 'waitlisted', 'rejected', 'confirmed', 'withdrawn')
    ),
    -- 접수한 직원과 시각(UTC, ISO 8601)
    submitted_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    submitted_at TEXT NOT NULL
) STRICT;

-- [BR-A4]: 같은 회원·강좌의 유효한 신청은 하나. 탈락·포기는 빠진다
CREATE UNIQUE INDEX applications_one_valid
    ON applications (member_id, course_id)
    WHERE status IN ('submitted', 'selected', 'waitlisted', 'confirmed');

-- 강좌별 신청 인원, 강좌가 쓰이는지 볼 때
CREATE INDEX applications_course ON applications (course_id);
-- 회원별 신청 목록, 회원이 쓰이는지 볼 때 (위 UNIQUE는 조건이 붙어 쓰이지 않음)
CREATE INDEX applications_member ON applications (member_id);

-- 규칙 예외. 신청 하나가 규칙 여러 개를 어기면 여러 줄 [BR-A6]
CREATE TABLE rule_overrides (
    id INTEGER PRIMARY KEY,
    -- 신청을 지우면(BR-D1) 함께 지운다. 지운 일은 삭제 기록에 남는다
    application_id INTEGER NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    -- [BR-A7]: 같은 강좌 두 번 신청(BR-A4)은 예외를 줄 수 없어서 여기 없다
    rule TEXT NOT NULL CHECK (rule IN ('no_time_overlap', 'no_same_subject')),
    reason TEXT NOT NULL CHECK (reason <> ''),
    staff_id INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    UNIQUE (application_id, rule)
) STRICT;

-- 규칙 예외는 고치지 않음
CREATE TRIGGER rule_overrides_no_update BEFORE UPDATE ON rule_overrides
BEGIN
    SELECT RAISE(ABORT, 'rule_overrides는 고칠 수 없음');
END;

-- 삭제 기록 [BR-D3]. 신청은 지워졌으므로 회원과 강좌를 가리킨다
CREATE TABLE application_deletions (
    id INTEGER PRIMARY KEY,
    -- [BR-M5], [BR-T6]: 삭제 기록만 남아도 회원·강좌를 지울 수 없음
    member_id INTEGER NOT NULL REFERENCES members (id) ON DELETE NO ACTION,
    course_id INTEGER NOT NULL REFERENCES courses (id) ON DELETE NO ACTION,
    deleted_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    deleted_at TEXT NOT NULL,
    -- 선택. 없으면 NULL (빈 글자로 두지 않음)
    reason TEXT CHECK (reason <> '')
) STRICT;

-- 회원·강좌를 지울 수 있는지 볼 때
CREATE INDEX application_deletions_member ON application_deletions (member_id);
CREATE INDEX application_deletions_course ON application_deletions (course_id);

-- 삭제 기록은 고치거나 지우지 않음
CREATE TRIGGER application_deletions_no_update BEFORE UPDATE ON application_deletions
BEGIN
    SELECT RAISE(ABORT, 'application_deletions는 고칠 수 없음');
END;

CREATE TRIGGER application_deletions_no_delete BEFORE DELETE ON application_deletions
BEGIN
    SELECT RAISE(ABORT, 'application_deletions는 지울 수 없음');
END;
