-- 학기. 학기마다 새로 만든다.
CREATE TABLE terms (
    id INTEGER PRIMARY KEY,
    -- [BR-T1]: 겹치지 않음
    name TEXT NOT NULL UNIQUE,
    -- YYYY-MM-DD. 글자 순서가 날짜 순서와 같음
    starts_on TEXT NOT NULL,
    ends_on TEXT NOT NULL,
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- [BR-T2]: 종료일은 시작일보다 앞서지 않음
    CHECK (ends_on >= starts_on)
) STRICT;

-- 강좌. 한 학기에 실제로 열려 신청을 받는 단위.
-- 쓰이는 학기·과목·강의실 지우기는 NO ACTION으로 막는다. RESTRICT는 SQLite가 다른 오류 코드(1811)를 내서
-- 외래 키 위반으로 알아보기 어렵다. 이 앱은 검사를 미루지 않으므로 막는 결과는 같다.
CREATE TABLE courses (
    id INTEGER PRIMARY KEY,
    -- [BR-T5]: 강좌가 있는 학기는 지울 수 없음
    term_id INTEGER NOT NULL REFERENCES terms (id) ON DELETE NO ACTION,
    -- [BR-J2]: 강좌가 쓰는 과목은 지울 수 없음
    subject_id INTEGER NOT NULL REFERENCES subjects (id) ON DELETE NO ACTION,
    -- 선택. 없으면 NULL (빈 글자로 두지 않음)
    level TEXT CHECK (level <> ''),
    section TEXT CHECK (section <> ''),
    capacity INTEGER NOT NULL CHECK (capacity BETWEEN 1 AND 200),
    -- UTC, ISO 8601
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

-- [BR-T3]: 같은 학기에 과목, 수준, 반이 같은 강좌는 하나.
-- UNIQUE는 NULL끼리를 다르다고 보므로, 비운 수준·반을 빈 글자로 바꿔 비교한다.
CREATE UNIQUE INDEX courses_unique_in_term
    ON courses (term_id, subject_id, IFNULL(level, ''), IFNULL(section, ''));

-- 강좌의 수업 시간. 강좌 하나에 한 줄 이상 (BR-T4, 줄 수와 줄끼리 겹침은 앱이 확인).
CREATE TABLE course_schedules (
    id INTEGER PRIMARY KEY,
    -- 강좌를 지우면 수업 시간도 함께 지움
    course_id INTEGER NOT NULL REFERENCES courses (id) ON DELETE CASCADE,
    -- 1 = 월요일 ... 7 = 일요일
    weekday INTEGER NOT NULL CHECK (weekday BETWEEN 1 AND 7),
    -- 자정부터 센 분 (10:00 = 600)
    start_minute INTEGER NOT NULL CHECK (start_minute BETWEEN 0 AND 1439),
    end_minute INTEGER NOT NULL CHECK (end_minute BETWEEN 0 AND 1439),
    -- [BR-R2]: 수업 시간이 쓰는 강의실은 지울 수 없음. 선택
    room_id INTEGER REFERENCES rooms (id) ON DELETE NO ACTION,
    -- [BR-T4]: 종료는 시작보다 늦음
    CHECK (end_minute > start_minute)
) STRICT;

-- 강좌의 수업 시간을 읽을 때, 강의실이 쓰이는지 볼 때
CREATE INDEX course_schedules_course ON course_schedules (course_id);
CREATE INDEX course_schedules_room ON course_schedules (room_id);
-- 학기의 강좌 목록, 과목이 쓰이는지 볼 때
CREATE INDEX courses_subject ON courses (subject_id);
