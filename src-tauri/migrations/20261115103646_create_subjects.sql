-- 과목. 학기가 바뀌어도 이어지고, 강좌를 열 때 고른다.
CREATE TABLE subjects (
    id INTEGER PRIMARY KEY,
    -- 직원이 쓴 띄어쓰기 그대로
    name TEXT NOT NULL,
    -- [BR-J1]: 이름에서 띄어쓰기를 모두 뺀 글자. 앱이 채움
    name_key TEXT NOT NULL UNIQUE CHECK (name_key <> ''),
    -- 선택. 없으면 NULL (빈 글자로 두지 않음)
    category TEXT CHECK (category <> ''),
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;
