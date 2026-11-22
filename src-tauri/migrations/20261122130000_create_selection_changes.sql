-- 선발 마감 뒤 포기, 대기자 연락 결과, 잘못 처리 바로잡기 [BR-C3, BR-W2~W8].
-- 신청 상태를 바꿀 때마다 한 줄씩 쌓는다. 고치거나 지우지 않는다 [BR-W7]
CREATE TABLE selection_changes (
    id INTEGER PRIMARY KEY,
    -- 마감 뒤 신청은 지울 수 없어(BR-D1) 외래 키로 가리킨다
    application_id INTEGER NOT NULL REFERENCES applications (id) ON DELETE NO ACTION,
    -- withdrawn: 포기, accepted/declined/unreachable: 연락 결과, corrected: 잘못 처리 바로잡기
    kind TEXT NOT NULL CHECK (
        kind IN ('withdrawn', 'accepted', 'declined', 'unreachable', 'corrected')
    ),
    -- 바뀌기 전과 뒤의 신청 상태. 연락 안 됨은 대기 그대로라 둘이 같다
    from_status TEXT NOT NULL CHECK (from_status IN ('selected', 'waitlisted', 'withdrawn')),
    to_status TEXT NOT NULL CHECK (to_status IN ('selected', 'waitlisted', 'withdrawn')),
    -- 포기는 선택, 바로잡기는 필수 [BR-W8]
    reason TEXT CHECK (reason IS NULL OR length(reason) > 0),
    changed_by INTEGER NOT NULL REFERENCES staff (id) ON DELETE NO ACTION,
    -- UTC, ISO 8601
    changed_at TEXT NOT NULL,
    CHECK (kind <> 'corrected' OR reason IS NOT NULL),
    -- 종류마다 가능한 상태 바뀜만
    CHECK (
        (kind = 'withdrawn' AND from_status IN ('selected', 'waitlisted') AND to_status = 'withdrawn')
        OR (kind = 'accepted' AND from_status = 'waitlisted' AND to_status = 'selected')
        OR (kind = 'declined' AND from_status = 'waitlisted' AND to_status = 'withdrawn')
        OR (kind = 'unreachable' AND from_status = 'waitlisted' AND to_status = 'waitlisted')
        OR (kind = 'corrected' AND from_status = 'withdrawn' AND to_status IN ('selected', 'waitlisted'))
    )
) STRICT;

-- 신청마다 마지막 연락 결과, 마지막 포기 전 상태를 찾을 때
CREATE INDEX selection_changes_application ON selection_changes (application_id, id);

CREATE TRIGGER selection_changes_no_update BEFORE UPDATE ON selection_changes
BEGIN
    SELECT RAISE(ABORT, 'selection_changes는 고칠 수 없음');
END;

CREATE TRIGGER selection_changes_no_delete BEFORE DELETE ON selection_changes
BEGIN
    SELECT RAISE(ABORT, 'selection_changes는 지울 수 없음');
END;
