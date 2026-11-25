-- 추가 신청 [BR-X1, BR-X2]. 선발 마감 뒤 받은 신청마다 한 줄.
-- 추첨 대상이 아니라 대기 순번을 추첨 기록에서 계산할 수 없어서, 받을 때 정한 순번을 둔다.
-- 고치거나 지우지 않는다. 선발 마감 뒤 신청은 지울 수 없다(BR-D1).
CREATE TABLE additional_applications (
    application_id INTEGER PRIMARY KEY REFERENCES applications (id) ON DELETE NO ACTION,
    -- 대기로 받았으면 그 대기 순번(BR-W6). 바로 당첨이면 NULL
    waitlist_number INTEGER CHECK (waitlist_number >= 1)
) STRICT;

CREATE TRIGGER additional_applications_no_update BEFORE UPDATE ON additional_applications
BEGIN
    SELECT RAISE(ABORT, 'additional_applications는 고칠 수 없음');
END;

CREATE TRIGGER additional_applications_no_delete BEFORE DELETE ON additional_applications
BEGIN
    SELECT RAISE(ABORT, 'additional_applications는 지울 수 없음');
END;
