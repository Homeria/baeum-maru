-- 결과 명단 내보내기 기록 [BR-P11].
-- 개인정보가 호스트 밖으로 나가는 순간이라 내보낼 때마다 한 줄씩 쌓는다. 고치거나 지우지 않는다.
-- 명단에 든 회원은 남기지 않는다 (기록이 또 하나의 명단이 되지 않게)
CREATE TABLE roster_exports (
    id INTEGER PRIMARY KEY,
    term_id INTEGER NOT NULL REFERENCES terms (id) ON DELETE NO ACTION,
    -- raw: 원본, posted_by_course: 게시용 강좌별, posted_by_member: 게시용 회원별
    kind TEXT NOT NULL CHECK (kind IN ('raw', 'posted_by_course', 'posted_by_member')),
    -- 넣은 칸과 가린 칸. 칸 이름을 쉼표로 이은 글자 (예: course_name,result,name,member_no)
    included_columns TEXT NOT NULL CHECK (length(included_columns) > 0),
    -- 가린 칸이 없으면 빈 글자
    masked_columns TEXT NOT NULL,
    mask_char TEXT NOT NULL CHECK (mask_char IN ('circle', 'asterisk', 'x')),
    -- 명단에 넣은 강좌 수 (선발을 마감한 강좌)
    course_count INTEGER NOT NULL CHECK (course_count > 0),
    exported_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    exported_at TEXT NOT NULL
) STRICT;

CREATE INDEX roster_exports_term ON roster_exports (term_id, id);

CREATE TRIGGER roster_exports_no_update BEFORE UPDATE ON roster_exports
BEGIN
    SELECT RAISE(ABORT, 'roster_exports는 고칠 수 없음');
END;

CREATE TRIGGER roster_exports_no_delete BEFORE DELETE ON roster_exports
BEGIN
    SELECT RAISE(ABORT, 'roster_exports는 지울 수 없음');
END;
