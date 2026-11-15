-- 회원. 학기가 바뀌어도 이어진다 (BR-M3).
CREATE TABLE members (
    id INTEGER PRIMARY KEY,
    -- [BR-M2]: 기관 안에서 겹치지 않음. 앞자리 0을 지키려고 글자로 저장
    member_no TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    -- 숫자만 (01012345678)
    phone_number TEXT NOT NULL,
    -- YYYY-MM-DD, 선택
    birth_date TEXT,
    -- 선택
    gender TEXT CHECK (gender IN ('male', 'female')),
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;
