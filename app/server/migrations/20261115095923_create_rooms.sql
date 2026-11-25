-- 강의실. 한 번 등록해 두고 강좌를 열 때 고른다.
CREATE TABLE rooms (
    id INTEGER PRIMARY KEY,
    -- [BR-R1]: 반드시 있음. 기본값 본관은 화면이 채움
    building TEXT NOT NULL,
    -- 지하 1층은 -1. 0층은 없음. 선택
    floor INTEGER CHECK (floor BETWEEN -2 AND 5 AND floor <> 0),
    name TEXT NOT NULL,
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- [BR-R1]: 같은 건물 안에서 이름이 겹치지 않음. 층은 보지 않음
    UNIQUE (building, name)
) STRICT;
