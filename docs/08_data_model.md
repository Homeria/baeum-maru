# 배움마루 데이터 모델

## 1. 설계 목표

이 문서는 배움마루의 초기 데이터 모델을 Boyce-Codd Normal Form, BCNF 기준으로 정리한다.

목표는 다음과 같다.

- 업무 개념을 명확히 분리한다.
- 중복 저장으로 인한 갱신 이상을 줄인다.
- 신청, 추첨, 대기자, 출석 흐름을 안정적으로 표현한다.
- SQLite에서 단순하게 구현할 수 있게 한다.
- 추후 PostgreSQL로 옮겨도 큰 구조 변경이 없게 한다.

## 2. BCNF 적용 기준

BCNF는 모든 비자명 함수 종속 `X -> Y`에서 `X`가 슈퍼키여야 한다는 정규화 조건이다.

배움마루에서는 다음 방식으로 적용한다.

- 코드명, 분류명, 강의실명처럼 반복되는 설명 값은 별도 테이블로 분리한다.
- 한 회원의 기본 정보는 `members`에만 둔다.
- 한 강좌의 기본 정의와 특정 접수 회차의 운영 정보를 분리한다.
- 신청 상태와 추첨 결과를 섞지 않는다.
- 로그와 이력은 현재 상태 테이블을 덮어쓰는 대신 별도 테이블에 남긴다.
- 보고서 편의를 위한 중복 컬럼은 우선 저장하지 않고 조회에서 조합한다.

## 3. 핵심 개념

```text
member
  수강 신청을 하는 사람

term
  접수와 강좌 운영이 묶이는 기간 또는 회차

course
  강좌의 기본 정의

course_offering
  특정 term에서 실제로 접수받는 강좌 개설 건

course_meeting
  강좌 개설 건의 요일/시간 정보

registration
  회원이 강좌 개설 건에 신청한 사실

lottery_run
  특정 시점에 실행한 추첨 작업

lottery_result
  추첨 실행 결과로 생긴 당첨/대기/탈락 기록
```

## 4. 테이블 목록

### 4.1 members

회원 기본 정보.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 내부 회원 ID |
| member_no | text | UNIQUE NULL 허용 | 기관 회원번호 |
| name | text | NOT NULL | 이름 |
| gender_code | text | FK -> gender_codes.code, NULL 허용 | 성별 코드 |
| birth_date | date | NULL 허용 | 생년월일 |
| phone | text | NULL 허용 | 연락처 |
| note | text | NULL 허용 | 비고 |
| created_at | datetime | NOT NULL | 등록일 |
| updated_at | datetime | NOT NULL | 수정일 |

BCNF 관점:

- `id -> member_no, name, gender_code, birth_date, phone, note`
- `member_no`가 존재하는 경우 `member_no -> id`
- 성별 표시명은 `gender_codes`로 분리한다.

### 4.2 gender_codes

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| code | text | PK | 예: male, female, unknown |
| label | text | NOT NULL UNIQUE | 표시명 |

### 4.3 terms

접수 회차 또는 학기.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 회차 ID |
| name | text | NOT NULL UNIQUE | 예: 2026년 여름학기 |
| registration_start_at | datetime | NULL 허용 | 접수 시작 |
| registration_end_at | datetime | NULL 허용 | 접수 종료 |
| max_registrations_per_member | integer | NOT NULL DEFAULT 0 | 0이면 제한 없음 |
| status | text | NOT NULL | draft, open, closed, finalized |
| created_at | datetime | NOT NULL | 생성일 |
| updated_at | datetime | NOT NULL | 수정일 |

### 4.4 course_categories

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 분류 ID |
| name | text | NOT NULL UNIQUE | 분류명 |
| sort_order | integer | NOT NULL DEFAULT 0 | 정렬 순서 |

### 4.5 courses

강좌의 기본 정의. 실제 접수 회차별 정원, 강사, 시간은 `course_offerings`와 `course_meetings`에 둔다.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 강좌 ID |
| title | text | NOT NULL | 강좌명 |
| category_id | integer | FK -> course_categories.id | 분류 |
| description | text | NULL 허용 | 설명 |
| is_active | boolean | NOT NULL DEFAULT true | 사용 여부 |
| created_at | datetime | NOT NULL | 생성일 |
| updated_at | datetime | NOT NULL | 수정일 |

BCNF 관점:

- `id -> title, category_id, description, is_active`
- `category_id -> category name` 종속은 `course_categories`로 분리한다.

### 4.6 instructors

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 강사 ID |
| name | text | NOT NULL | 강사명 |
| phone | text | NULL 허용 | 연락처 |
| note | text | NULL 허용 | 비고 |
| created_at | datetime | NOT NULL | 생성일 |
| updated_at | datetime | NOT NULL | 수정일 |

### 4.7 classrooms

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 강의실 ID |
| name | text | NOT NULL UNIQUE | 강의실명 |
| note | text | NULL 허용 | 비고 |

### 4.8 course_offerings

특정 회차에서 실제 접수받는 강좌 개설 건.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 강좌 개설 ID |
| term_id | integer | FK -> terms.id | 회차 |
| course_id | integer | FK -> courses.id | 강좌 |
| instructor_id | integer | FK -> instructors.id, NULL 허용 | 강사 |
| classroom_id | integer | FK -> classrooms.id, NULL 허용 | 강의실 |
| capacity | integer | NOT NULL CHECK capacity >= 0 | 정원 |
| registration_enabled | boolean | NOT NULL DEFAULT true | 접수 가능 여부 |
| status | text | NOT NULL | draft, open, closed, cancelled |
| note | text | NULL 허용 | 비고 |
| created_at | datetime | NOT NULL | 생성일 |
| updated_at | datetime | NOT NULL | 수정일 |

권장 제약:

```text
UNIQUE(term_id, course_id)
```

같은 회차에 같은 강좌 정의가 두 번 열릴 수 있다면 `section_name` 컬럼을 추가하고 `UNIQUE(term_id, course_id, section_name)`으로 바꾼다.

### 4.9 course_meetings

강좌 시간 정보. 한 강좌가 주 2회 이상 열릴 수 있으므로 분리한다.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 시간표 ID |
| offering_id | integer | FK -> course_offerings.id | 강좌 개설 |
| weekday | integer | NOT NULL CHECK 0~6 | 요일 |
| start_time | text | NOT NULL | HH:MM |
| end_time | text | NOT NULL | HH:MM |

권장 제약:

```text
CHECK(start_time < end_time)
UNIQUE(offering_id, weekday, start_time, end_time)
```

### 4.10 registrations

회원의 신청 사실.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 신청 ID |
| term_id | integer | FK -> terms.id | 회차 |
| member_id | integer | FK -> members.id | 회원 |
| offering_id | integer | FK -> course_offerings.id | 강좌 개설 |
| status | text | NOT NULL | applied, cancelled, selected, waitlisted, rejected, confirmed |
| created_at | datetime | NOT NULL | 신청일 |
| updated_at | datetime | NOT NULL | 수정일 |
| cancelled_at | datetime | NULL 허용 | 취소일 |

권장 제약:

```text
UNIQUE(member_id, offering_id)
```

BCNF 관점:

- `id -> term_id, member_id, offering_id, status, created_at, updated_at`
- `member_id, offering_id -> id`는 중복 신청 방지용 후보키다.
- `term_id`는 `offering_id -> term_id`로 유도 가능하지만, 조회와 제약 확인을 쉽게 하기 위해 저장할 수 있다. 엄격한 BCNF만 보면 중복이므로, 초기 구현에서는 둘 중 하나를 선택한다.

권장:

- 순수 BCNF를 우선하면 `term_id`를 제거한다.
- SQLite 쿼리 단순성을 우선하면 `term_id`를 두되, 서비스 계층에서 `offering.term_id`와 일치하도록 강제한다.

초기 구현 권장안은 `term_id`를 제거하는 것이다.

### 4.11 registration_status_history

신청 상태 변경 이력.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 이력 ID |
| registration_id | integer | FK -> registrations.id | 신청 |
| from_status | text | NULL 허용 | 이전 상태 |
| to_status | text | NOT NULL | 새 상태 |
| reason | text | NULL 허용 | 변경 사유 |
| changed_by_user_id | integer | FK -> users.id, NULL 허용 | 변경자 |
| changed_at | datetime | NOT NULL | 변경일 |

### 4.12 lottery_runs

추첨 실행 단위.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 추첨 실행 ID |
| term_id | integer | FK -> terms.id | 회차 |
| seed | integer | NOT NULL | 재현용 seed |
| status | text | NOT NULL | prepared, completed, cancelled |
| started_at | datetime | NOT NULL | 시작 시각 |
| completed_at | datetime | NULL 허용 | 완료 시각 |
| executed_by_user_id | integer | FK -> users.id, NULL 허용 | 실행자 |
| note | text | NULL 허용 | 비고 |

### 4.13 lottery_results

추첨 결과. 정원 이하 자동 확정도 같은 구조에 기록할 수 있다.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 결과 ID |
| lottery_run_id | integer | FK -> lottery_runs.id | 추첨 실행 |
| registration_id | integer | FK -> registrations.id | 신청 |
| result | text | NOT NULL | selected, waitlisted, rejected |
| result_order | integer | NOT NULL | 추첨 순서 또는 대기 순서 |
| created_at | datetime | NOT NULL | 생성일 |

권장 제약:

```text
UNIQUE(lottery_run_id, registration_id)
```

대기자 순번은 `result = waitlisted`인 행의 `result_order`로 표현한다. 별도 `waitlists` 테이블은 MVP에서 만들지 않는다.

### 4.14 attendance_sessions

강좌별 출석일.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 출석일 ID |
| offering_id | integer | FK -> course_offerings.id | 강좌 개설 |
| session_date | date | NOT NULL | 수업일 |
| note | text | NULL 허용 | 비고 |

권장 제약:

```text
UNIQUE(offering_id, session_date)
```

### 4.15 attendance_records

회원별 출석 기록.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 출석 기록 ID |
| attendance_session_id | integer | FK -> attendance_sessions.id | 출석일 |
| registration_id | integer | FK -> registrations.id | 수강 신청 |
| status | text | NOT NULL | present, absent, late, excused |
| note | text | NULL 허용 | 비고 |

권장 제약:

```text
UNIQUE(attendance_session_id, registration_id)
```

### 4.16 users

웹 업무 사용자의 감사용 사용자 기록이다. 계정 영구 로그인보다는 런처에서 발급한 접속 코드와 연결되는 사용자 정보를 유지한다.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 사용자 ID |
| username | text | NOT NULL UNIQUE | 내부 식별자 |
| password_hash | text | NOT NULL | 레거시 호환 자리, 접속 코드 사용자는 고정 placeholder |
| display_name | text | NOT NULL | 표시명 |
| role | text | NOT NULL | 레거시 호환 역할 |
| is_active | boolean | NOT NULL DEFAULT true | 활성 여부 |
| affiliation | text | NULL 허용 | 소속 또는 현장 메모 |
| contact_note | text | NULL 허용 | 감사 추적용 연락/식별 메모 |
| access_role | text | NOT NULL | staff, temporary_staff, viewer |
| status | text | NOT NULL | active, expired, disabled |
| expires_at | datetime | NULL 허용 | 사용자 기록 만료 기준 |
| last_login_at | datetime | NULL 허용 | 마지막 로그인 |
| created_at | datetime | NOT NULL | 생성일 |
| updated_at | datetime | NOT NULL | 수정일 |

### 4.17 access_codes

런처에서 발급한 접속 코드의 검증 정보이다. 원문 코드는 저장하지 않고 해시만 저장한다.

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 접속 코드 ID |
| user_id | integer | FK -> users.id | 코드 소유 사용자 |
| code_hash | text | NOT NULL UNIQUE | 접속 코드 해시 |
| label | text | NULL 허용 | 발급 구분명 |
| status | text | NOT NULL | active, expired, revoked |
| issued_at | datetime | NOT NULL | 발급 시각 |
| expires_at | datetime | NOT NULL | 만료 시각 |
| revoked_at | datetime | NULL 허용 | 폐기 시각 |
| last_used_at | datetime | NULL 허용 | 마지막 사용 시각 |
| note | text | NULL 허용 | 메모 |
| created_at | datetime | NOT NULL | 생성일 |
| updated_at | datetime | NOT NULL | 수정일 |

### 4.18 audit_logs

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| id | integer | PK | 로그 ID |
| actor_user_id | integer | FK -> users.id, NULL 허용 | 수행자 |
| action | text | NOT NULL | 작업명 |
| entity_type | text | NOT NULL | 대상 종류 |
| entity_id | integer | NULL 허용 | 대상 ID |
| summary | text | NOT NULL | 사용자 친화 요약 |
| created_at | datetime | NOT NULL | 시각 |

주의:

- 전화번호, 생년월일 등 개인정보를 summary에 직접 남기지 않는다.
- 상세 디버그 로그와 감사 로그를 분리한다.

### 4.19 settings

| 컬럼 | 타입 | 제약 | 설명 |
|---|---|---|---|
| key | text | PK | 설정 키 |
| value | text | NOT NULL | 설정 값 |
| updated_at | datetime | NOT NULL | 수정일 |

## 5. 관계 요약

```text
terms 1 ─ N course_offerings
courses 1 ─ N course_offerings
course_categories 1 ─ N courses
instructors 1 ─ N course_offerings
classrooms 1 ─ N course_offerings
course_offerings 1 ─ N course_meetings
members 1 ─ N registrations
course_offerings 1 ─ N registrations
registrations 1 ─ N registration_status_history
terms 1 ─ N lottery_runs
lottery_runs 1 ─ N lottery_results
registrations 1 ─ N lottery_results
course_offerings 1 ─ N attendance_sessions
attendance_sessions 1 ─ N attendance_records
registrations 1 ─ N attendance_records
users 1 ─ N access_codes
users 1 ─ N audit_logs
```

## 6. 상태값

### term.status

```text
draft
open
closed
finalized
```

### course_offerings.status

```text
draft
open
closed
cancelled
```

### registrations.status

```text
applied
cancelled
selected
waitlisted
rejected
confirmed
```

### lottery_results.result

```text
selected
waitlisted
rejected
```

## 7. 인덱스 권장

```sql
CREATE INDEX idx_members_name ON members(name);
CREATE INDEX idx_members_phone ON members(phone);
CREATE INDEX idx_courses_title ON courses(title);
CREATE INDEX idx_course_offerings_term ON course_offerings(term_id);
CREATE INDEX idx_registrations_member ON registrations(member_id);
CREATE INDEX idx_registrations_offering ON registrations(offering_id);
CREATE INDEX idx_registrations_status ON registrations(status);
CREATE INDEX idx_lottery_results_run ON lottery_results(lottery_run_id);
```

## 8. MVP에서 단순화 가능한 부분

초기 구현 속도를 위해 다음은 단순화할 수 있다.

- `instructors`를 만들지 않고 강사명을 `course_offerings.instructor_name`에 둘 수 있다.
- `classrooms`를 만들지 않고 강의실명을 `course_offerings.classroom_name`에 둘 수 있다.
- `attendance_*` 테이블은 엑셀 출석부 출력만 필요하면 나중에 만든다.
- 접속 코드 방식에서는 사용자를 삭제하지 않고 `status = expired` 또는 접속 코드 만료로 남겨 감사 로그의 수행자를 추적한다.

다만 BCNF 학습 목적을 유지하려면 분리된 모델을 먼저 이해한 뒤, 의도적인 단순화인지 기록하고 진행한다.

## 9. 구현 순서 권장

1. `terms`
2. `members`
3. `course_categories`
4. `courses`
5. `course_offerings`
6. `course_meetings`
7. `registrations`
8. `registration_status_history`
9. `lottery_runs`
10. `lottery_results`
11. `users`
12. `access_codes`
13. `audit_logs`
14. `settings`
15. `attendance_sessions`
16. `attendance_records`
