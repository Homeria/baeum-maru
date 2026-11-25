-- 데이터 묶음 ID와 세대 번호(BR-H4, ADR 0002). 한 줄만 둔다.
-- 이미 쓰던 DB도 이 구조 변경 때 무작위 묶음 ID와 세대 1을 받는다.
CREATE TABLE data_bundle (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    bundle_id TEXT NOT NULL
        CHECK (length(bundle_id) = 32 AND bundle_id NOT GLOB '*[^0-9a-f]*'),
    generation INTEGER NOT NULL CHECK (generation >= 1)
);

INSERT INTO data_bundle (id, bundle_id, generation)
VALUES (1, lower(hex(randomblob(16))), 1);

-- 줄이 없어지면 접속 PC가 호스트를 알아보지 못한다
CREATE TRIGGER data_bundle_no_delete
BEFORE DELETE ON data_bundle
BEGIN
    SELECT RAISE(ABORT, '데이터 묶음 정보는 지울 수 없습니다');
END;
