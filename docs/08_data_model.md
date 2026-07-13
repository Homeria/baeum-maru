# Python 초기 스키마 기준선

## 상태와 범위

이 문서는 Python 프로토타입의 단일 초기 SQLAlchemy/Alembic 스키마 설계 명세다. `app/models/`와 `20260713_0001_initial_schema` revision이 이 기준을 구현하며 이후 변경도 이 문서와 함께 갱신한다.

- 과거 Go DB와의 호환 migration은 작성하지 않는다.
- 초기 Alembic revision 이후 실제 데이터가 생기면 모든 변경을 migration으로 수행한다.
- 재구현 이후에는 이 문서, SQLAlchemy model, Alembic revision, SQLite contract test가 함께 일치해야 한다.
- 현재 범위는 기관 한 곳과 SQLite 파일 하나다. 모든 테이블에 tenant/organization FK를 반복하지 않는다.

## 확정한 모델링 결정

1. `locations`는 `building_floor_id`를 필수 FK로 가진다. `building_id`와 `floor_label`을 중복 저장하지 않는다.
2. 층은 숫자가 아니라 `building_floors.label` 문자열이다. `지하 1층`, `옥상`, `별관 연결층`을 지원한다.
3. `location_roles`는 사용자가 추가·수정할 수 있고 공간에 다중 지정한다. 강좌 선택 가능 여부는 역할 이름이 아니라 `is_course_eligible`로 판단한다.
4. `courses`는 회차와 무관한 과목 원형이다. `한글교실 초급`처럼 난이도가 정체성에 가까우면 과목명에 포함한다.
5. `course_offerings.section_label`만 선택적으로 둔다. `컴퓨터초급 + 1반`처럼 같은 과목의 분반을 표현하며 별도 `level_label`은 두지 않는다.
6. 공간은 `course_offerings`가 아니라 각 `course_schedules`가 참조한다. 같은 개설 강좌가 요일별로 다른 공간을 쓸 수 있다.
7. 직원 로그인은 접속 코드 방식만 사용한다. `users`는 행위자 프로필이며 username/password를 저장하지 않는다.
8. 접속 코드와 세션의 `expired` 상태는 `expires_at`에서 계산한다. 만료 상태를 별도 저장해 시간과 불일치시키지 않는다.
9. 임시 접속 코드 원문은 런처 목록 재확인을 위해 `display_code`로 보관한다. 인증 비교는 hash를 사용하고 원문 조회는 localhost pywebview bridge에만 허용한다.
10. 신청의 현재 상태와 상태 이력은 의도적으로 함께 저장한다. 현재 조회 성능과 감사 이력이라는 서로 다른 책임이다.
11. 추첨 대상의 정원과 인원은 실행 당시 snapshot으로 남긴다. 이는 재현과 감사에 필요한 의도적 비정규화다.
12. 범용 EAV 설정 테이블은 두지 않는다. 기관명, 로고와 운영 기본값은 singleton `organization_settings`에 명시적으로 둔다.

## 공통 타입과 규칙

| 논리 타입 | SQLAlchemy | SQLite 저장 | 규칙 |
|---|---|---|---|
| PK | `Integer` | `INTEGER` | 업무 테이블은 자동 증가 정수 ID |
| 짧은 문자열 | `String(n)` | `TEXT` | API에서 trim/길이 검증 |
| 긴 문자열 | `Text` | `TEXT` | 메모, 요약, 오류 |
| bool | `Boolean` | `INTEGER` | `0/1` CHECK 포함 |
| 날짜 | `Date` | ISO `TEXT` | `YYYY-MM-DD` |
| 시간 | `Time` | ISO `TEXT` | `HH:MM[:SS]` |
| 시각 | UTC datetime | ISO `TEXT` | 입력 시 UTC 정규화, 응답은 timezone 포함 |
| JSON | `JSON` | `TEXT` | object만 허용하고 Pydantic으로 검증 |

변경 가능한 기준/업무 엔티티는 `created_at`, `updated_at`, `version`을 가진다. `version`은 1부터 시작하며 수정 SQL의 WHERE 조건에 포함해 optimistic concurrency를 강제한다. 단순 관계, append-only 이력, worker 상태에는 불필요한 version을 두지 않는다.

이름과 회원번호는 application에서 trim한 값을 저장한다. 전화번호는 검색과 Excel 일관성을 위해 숫자만 저장하고 화면에서 포맷한다. 주민등록번호와 생년월일은 프로토타입 필수 정보가 아니므로 저장하지 않는다.

## 테이블 목록

| 영역 | 테이블 |
|---|---|
| 기관 | `organization_settings` |
| 인증 | `users`, `access_codes`, `user_sessions` |
| 공간 | `buildings`, `building_floors`, `locations`, `location_roles`, `location_role_assignments` |
| 강좌 기준 | `course_categories`, `courses`, `instructors`, `terms`, `time_slots` |
| 강좌 운영 | `course_offerings`, `course_schedules` |
| 회원/접수 | `gender_codes`, `members`, `registrations`, `registration_status_history` |
| 추첨 | `lottery_runs`, `lottery_run_targets`, `lottery_results` |
| 출석 | `attendance_sessions`, `attendance_records` |
| 운영 이력 | `audit_logs`, `operation_jobs`, `operation_job_errors` |
| 동시성 | `idempotency_records`, `operation_locks` |

## 기관과 인증

### `organization_settings`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK, `CHECK (id = 1)` singleton |
| `organization_name` | VARCHAR(120) | NOT NULL | 화면과 문서에 표시할 기관명 |
| `logo_relative_path` | VARCHAR(260) | NULL | runtime 기준 상대 경로, 절대 경로 금지 |
| `default_max_registrations` | INTEGER | NOT NULL / 4 | `>= 0`, 0은 제한 없음 |
| `default_access_code_ttl_minutes` | INTEGER | NOT NULL / 480 | `1..10080` |
| `created_at`, `updated_at` | DATETIME | NOT NULL | UTC |
| `version` | INTEGER | NOT NULL / 1 | `>= 1` |

첫 실행 설정을 완료할 때 id 1을 생성한다. migration은 실제 기관명을 seed하지 않는다.

### `users`

접속 코드를 발급받는 직원/자원봉사자의 감사용 프로필이다. 영구 비밀번호 계정이 아니다.

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `display_name` | VARCHAR(80) | NOT NULL | 실제 행위자 표시명 |
| `affiliation` | VARCHAR(120) | NULL | 부서, 소속, 자원봉사 구분 |
| `contact_note` | VARCHAR(120) | NULL | 필요한 최소 연락 메모, 전화번호 강제 안 함 |
| `role` | VARCHAR(24) | NOT NULL | `staff`, `temporary_staff`, `viewer` |
| `is_active` | BOOLEAN | NOT NULL / true | 비활성 사용자는 로그인 불가 |
| `created_at`, `updated_at` | DATETIME | NOT NULL | UTC |
| `version` | INTEGER | NOT NULL / 1 | optimistic concurrency |

동명이인을 허용하므로 이름은 unique가 아니다. 사용자는 감사 FK가 존재하면 hard delete하지 않고 비활성화한다.

### `access_codes`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `user_id` | INTEGER | NOT NULL | FK `users.id` RESTRICT |
| `code_hash` | VARCHAR(255) | NOT NULL | UNIQUE, 로그인 비교용 |
| `display_code` | VARCHAR(32) | NOT NULL | UNIQUE, 로컬 런처 재확인용 원문, LAN API 노출 금지 |
| `label` | VARCHAR(120) | NULL | 발급 목적/근무일 설명 |
| `issued_at` | DATETIME | NOT NULL | UTC |
| `expires_at` | DATETIME | NOT NULL | `expires_at > issued_at` |
| `revoked_at` | DATETIME | NULL | NULL이면 미폐기 |
| `hidden_at` | DATETIME | NULL | 런처의 목록 삭제는 soft hide로 처리 |
| `last_used_at` | DATETIME | NULL | 마지막 성공 인증 시각 |
| `note` | TEXT | NULL | 운영 메모 |
| `created_at`, `updated_at` | DATETIME | NOT NULL | UTC |
| `version` | INTEGER | NOT NULL / 1 | 연장/폐기 충돌 검사용 |

표시 상태는 다음처럼 계산한다.

- 사용 가능: `revoked_at IS NULL AND expires_at > now AND hidden_at IS NULL`이며 user 활성
- 만료: `revoked_at IS NULL AND expires_at <= now`
- 폐기: `revoked_at IS NOT NULL`
- 목록 제외: `hidden_at IS NOT NULL`; 감사/세션 FK 때문에 row는 유지

`display_code` 보관은 호스트 담당자가 발급 코드를 다시 확인해야 한다는 제품 요구에 따른 의도적 보안 절충이다. 직원용 REST API, WebSocket, 일반 로그, 감사 metadata와 Excel에는 절대 포함하지 않으며 런처의 로컬 bridge만 반환한다. 코드는 짧은 만료 시간과 즉시 폐기를 전제로 한다.

### `user_sessions`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | VARCHAR(36) | NOT NULL | UUID PK, cookie에 직접 노출하지 않음 |
| `user_id` | INTEGER | NOT NULL | FK `users.id` RESTRICT |
| `access_code_id` | INTEGER | NOT NULL | FK `access_codes.id` RESTRICT |
| `token_hash` | VARCHAR(255) | NOT NULL | UNIQUE |
| `issued_at` | DATETIME | NOT NULL | UTC |
| `expires_at` | DATETIME | NOT NULL | `expires_at > issued_at` |
| `last_seen_at` | DATETIME | NOT NULL | presence 계산용 |
| `revoked_at` | DATETIME | NULL | logout/강제 종료 |

세션 활성 여부도 만료 시각과 `revoked_at`에서 계산한다. 프로세스 재시작 후 DB만으로 인증 상태를 판정한다.

## 공간 기준 데이터

### `buildings`

`id` PK, `name VARCHAR(120) NOT NULL UNIQUE`, `description TEXT NULL`, `sort_order INTEGER NOT NULL DEFAULT 0`, `is_active BOOLEAN NOT NULL DEFAULT true`, 공통 `created_at/updated_at/version`을 가진다.

### `building_floors`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `building_id` | INTEGER | NOT NULL | FK `buildings.id` CASCADE |
| `label` | VARCHAR(80) | NOT NULL | 숫자로 제한하지 않음 |
| `sort_order` | INTEGER | NOT NULL / 0 | 지하/지상 표시 순서 |
| `is_active` | BOOLEAN | NOT NULL / true | 비활성 층 신규 선택 금지 |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 변경 충돌 검사용 |

`UNIQUE (building_id, label)`. 건물 hard delete 시 소속 층은 cascade되지만, 층을 참조하는 공간이 하나라도 있으면 `locations`의 RESTRICT가 전체 삭제를 막는다.

### `location_roles`

`id` PK, `name VARCHAR(80) NOT NULL UNIQUE`, `is_course_eligible BOOLEAN NOT NULL DEFAULT false`, `sort_order INTEGER NOT NULL DEFAULT 0`, `is_active BOOLEAN NOT NULL DEFAULT true`, 공통 변경 컬럼을 가진다. 역할 이름은 완전히 사용자 정의이며 초기 migration에서 강의/사무 같은 값을 강제 seed하지 않는다.

### `locations`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `building_floor_id` | INTEGER | NOT NULL | FK `building_floors.id` RESTRICT |
| `name` | VARCHAR(120) | NOT NULL | 예: 문화교육실, 대강당 |
| `description` | TEXT | NULL | 고정적인 공간 설명만 저장 |
| `sort_order` | INTEGER | NOT NULL / 0 | 목록 표시 순서 |
| `is_active` | BOOLEAN | NOT NULL / true | 비활성 공간 신규 일정 지정 금지 |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 변경 충돌 검사용 |

`UNIQUE (building_floor_id, name)`. 건물과 층 표시는 FK join으로 읽으며 공지 문구나 행사 상태는 저장하지 않는다.

건물에 속하지 않는 야외/임시 공간이 필요하면 FK를 NULL로 만들지 않고 `외부/기타` 건물과 `해당 없음` 층을 기준 데이터로 등록한다. 같은 선택/필터 구조를 유지하면서 예외 공간도 표현할 수 있다.

### `location_role_assignments`

`location_id FK locations.id CASCADE`, `role_id FK location_roles.id CASCADE`의 복합 PK다. 역할 순서는 `location_roles.sort_order`를 사용한다.

## 강좌 기준과 개설

### `course_categories`

`id` PK, `name VARCHAR(100) NOT NULL UNIQUE`, `description TEXT NULL`, `sort_order INTEGER NOT NULL DEFAULT 0`, `is_active BOOLEAN NOT NULL DEFAULT true`, 공통 변경 컬럼을 가진다. 예: 평생교육, 취미여가.

### `courses`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `category_id` | INTEGER | NOT NULL | FK `course_categories.id` RESTRICT |
| `name` | VARCHAR(160) | NOT NULL | 예: 한글교실 초급, 컴퓨터초급 |
| `description` | TEXT | NULL | 과목 자체 설명 |
| `is_active` | BOOLEAN | NOT NULL / true | 신규 개설 선택 여부 |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 변경 충돌 검사용 |

`UNIQUE (category_id, name)`. 정원, 회차, 시간, 공간, 강사와 분반은 저장하지 않는다.

### `instructors`

`id` PK, `name VARCHAR(80) NOT NULL`, `phone VARCHAR(20) NULL`, `note TEXT NULL`, `is_active BOOLEAN NOT NULL DEFAULT true`, 공통 변경 컬럼을 가진다. 동명이인을 허용하므로 이름은 unique가 아니다.

### `terms`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `name` | VARCHAR(120) | NOT NULL | UNIQUE, 예: 2026년 2학기 |
| `starts_on`, `ends_on` | DATE | NULL | 둘 다 있으면 `starts_on <= ends_on` |
| `registration_opens_at` | DATETIME | NULL | 접수 시작 예정 시각 |
| `registration_closes_at` | DATETIME | NULL | 둘 다 있으면 시작보다 늦어야 함 |
| `max_registrations_per_member` | INTEGER | NOT NULL / 0 | `>= 0`, 0은 제한 없음 |
| `status` | VARCHAR(16) | NOT NULL / `draft` | `draft`, `open`, `closed`, `finalized` |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 변경 충돌 검사용 |

기관 기본 신청 개수는 회차 생성 시 복사한다. 이후 기관 기본값이 바뀌어도 기존 회차 정책은 바뀌지 않는다.

### `time_slots`

`id` PK, `name VARCHAR(80) NOT NULL UNIQUE`, `start_time TIME NOT NULL`, `end_time TIME NOT NULL`, `sort_order INTEGER NOT NULL DEFAULT 0`, `is_active BOOLEAN NOT NULL DEFAULT true`, 공통 변경 컬럼을 가진다. `CHECK (start_time < end_time)`와 `UNIQUE (start_time, end_time)`를 둔다.

### `course_offerings`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `term_id` | INTEGER | NOT NULL | FK `terms.id` RESTRICT |
| `course_id` | INTEGER | NOT NULL | FK `courses.id` RESTRICT |
| `section_label` | VARCHAR(80) | NULL | 예: 1반, 2반 |
| `instructor_id` | INTEGER | NULL | FK `instructors.id` RESTRICT |
| `capacity_type` | VARCHAR(20) | NOT NULL / `fixed` | `fixed`, `open`, `gender_split` |
| `capacity_total` | INTEGER | NULL | fixed일 때 `> 0` |
| `male_capacity`, `female_capacity` | INTEGER | NULL | gender_split일 때 각각 `>= 0`, 합계 `> 0` |
| `status` | VARCHAR(16) | NOT NULL / `draft` | `draft`, `open`, `closed`, `cancelled` |
| `sort_order` | INTEGER | NOT NULL / 0 | 시간표/목록 순서 |
| `note` | TEXT | NULL | 회차별 메모 |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 변경 충돌 검사용 |

정원 CHECK는 다음을 강제한다.

- fixed: `capacity_total > 0`, 성별 정원 NULL
- open: 모든 정원 NULL
- gender_split: total NULL, 남/여 정원 NOT NULL 및 합계 `> 0`

SQLite의 NULL unique 동작을 피하기 위해 두 partial unique index를 둔다.

- `UNIQUE (term_id, course_id) WHERE section_label IS NULL`
- `UNIQUE (term_id, course_id, section_label) WHERE section_label IS NOT NULL`

빈 분반 문자열은 저장 전에 NULL로 정규화한다.

### `course_schedules`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `offering_id` | INTEGER | NOT NULL | FK `course_offerings.id` CASCADE |
| `weekday` | INTEGER | NOT NULL | ISO 1(월)..7(일) |
| `time_slot_id` | INTEGER | NOT NULL | FK `time_slots.id` RESTRICT |
| `location_id` | INTEGER | NOT NULL | FK `locations.id` RESTRICT |

`UNIQUE (offering_id, weekday, time_slot_id)`. 한 개설 강좌 내부의 중복 행은 DB가 막고, 서로 다른 time slot의 실제 시간 겹침은 application이 검사한다.

## 회원과 접수

### `gender_codes`

`code VARCHAR(16) PK`, `label VARCHAR(40) NOT NULL UNIQUE`, `sort_order INTEGER NOT NULL DEFAULT 0`이다. initial migration은 `male`, `female`, `unknown`을 seed한다. 임의 삭제/수정 UI는 제공하지 않는다.

### `members`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `member_no` | VARCHAR(40) | NOT NULL | UNIQUE, 기관 회원번호 |
| `name` | VARCHAR(80) | NOT NULL | 검색 대상 |
| `gender_code` | VARCHAR(16) | NOT NULL / `unknown` | FK `gender_codes.code` RESTRICT |
| `phone` | VARCHAR(20) | NOT NULL | 숫자만 저장, 검색 대상 |
| `note` | TEXT | NULL | 최소한의 업무 메모 |
| `is_active` | BOOLEAN | NOT NULL / true | 탈퇴/이용중지 회원 신규 신청 금지 |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 변경 충돌 검사용 |

이름과 전화번호 중복은 허용한다. 회원 병합은 별도 업무 규칙으로 다루며 DB unique로 추측하지 않는다.

### `registrations`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `member_id` | INTEGER | NOT NULL | FK `members.id` RESTRICT |
| `offering_id` | INTEGER | NOT NULL | FK `course_offerings.id` RESTRICT |
| `status` | VARCHAR(16) | NOT NULL / `applied` | `applied`, `selected`, `waitlisted`, `rejected`, `confirmed`, `cancelled` |
| `cancelled_at` | DATETIME | NULL | cancelled일 때만 NOT NULL |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 현재 상태와 충돌 검사용 |

`UNIQUE (member_id, offering_id)`. 취소 후 재신청은 새 row를 만들지 않고 같은 신청을 다시 전이하며 이력에 남긴다.

### `registration_status_history`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `registration_id` | INTEGER | NOT NULL | FK `registrations.id` CASCADE |
| `from_status` | VARCHAR(16) | NULL | 최초 생성만 NULL |
| `to_status` | VARCHAR(16) | NOT NULL | registration status enum |
| `reason` | VARCHAR(255) | NULL | 전이 사유 |
| `actor_kind` | VARCHAR(16) | NOT NULL | `user`, `launcher`, `system` |
| `actor_user_id` | INTEGER | NULL | FK `users.id` RESTRICT |
| `actor_access_code_id` | INTEGER | NULL | FK `access_codes.id` RESTRICT |
| `actor_display_name` | VARCHAR(80) | NULL | 당시 표시명 snapshot |
| `metadata_json` | JSON | NULL | 구조화된 부가 정보 |
| `changed_at` | DATETIME | NOT NULL | UTC |

신청 생성/상태 변경 transaction은 현재 row와 history를 반드시 함께 기록한다.

## 추첨

### `lottery_runs`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `term_id` | INTEGER | NOT NULL | FK `terms.id` RESTRICT |
| `seed` | INTEGER | NOT NULL | 재현 가능한 난수 seed |
| `status` | VARCHAR(16) | NOT NULL / `prepared` | `prepared`, `running`, `completed`, `failed`, `cancelled` |
| `executed_by_user_id` | INTEGER | NOT NULL | FK `users.id` RESTRICT |
| `created_at` | DATETIME | NOT NULL | 준비 시각 |
| `started_at`, `completed_at` | DATETIME | NULL | 실행 lifecycle |
| `note` | TEXT | NULL | 운영 메모 |

### `lottery_run_targets`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `lottery_run_id` | INTEGER | NOT NULL | FK `lottery_runs.id` CASCADE |
| `offering_id` | INTEGER | NOT NULL | FK `course_offerings.id` RESTRICT |
| `capacity_type` | VARCHAR(20) | NOT NULL | 실행 당시 snapshot |
| `capacity_total` | INTEGER | NULL | snapshot |
| `male_capacity`, `female_capacity` | INTEGER | NULL | snapshot |
| `eligible_count` | INTEGER | NOT NULL | `>= 0`, 실행 당시 신청 수 |

`UNIQUE (lottery_run_id, offering_id)`이며 offering 하나는 같은 run에 한 번만 포함된다. 정원 CHECK는 `course_offerings`와 동일하다.

### `lottery_results`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `lottery_run_target_id` | INTEGER | NOT NULL | FK `lottery_run_targets.id` CASCADE |
| `registration_id` | INTEGER | NOT NULL | FK `registrations.id` RESTRICT |
| `result` | VARCHAR(16) | NOT NULL | `selected`, `waitlisted`, `rejected` |
| `result_order` | INTEGER | NOT NULL | `>= 1`, 결과 그룹 내 순번 |
| `created_at` | DATETIME | NOT NULL | UTC |

`UNIQUE (lottery_run_target_id, registration_id)`와 `UNIQUE (lottery_run_target_id, result, result_order)`를 둔다. registration이 target의 offering에 속하고 같은 run의 다른 target에 중복되지 않는지는 실행 transaction에서 검증한다.

## 출석

### `attendance_sessions`

`id` PK, `offering_id FK course_offerings.id RESTRICT`, `schedule_id FK course_schedules.id RESTRICT NULL`, `session_date DATE NOT NULL`, `sequence_no INTEGER NOT NULL DEFAULT 1 CHECK >= 1`, `note TEXT NULL`, 공통 변경 컬럼을 가진다. `UNIQUE (offering_id, session_date, sequence_no)`.

### `attendance_records`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `attendance_session_id` | INTEGER | NOT NULL | FK `attendance_sessions.id` CASCADE |
| `registration_id` | INTEGER | NOT NULL | FK `registrations.id` RESTRICT |
| `status` | VARCHAR(16) | NOT NULL | `present`, `absent`, `late`, `excused` |
| `note` | TEXT | NULL | 출석 메모 |
| `created_at`, `updated_at`, `version` | 공통 | NOT NULL | 수정 충돌 검사용 |

`UNIQUE (attendance_session_id, registration_id)`. 신청이 해당 session의 offering에 속하는지는 application이 검사한다.

## 감사와 장기 작업

### `audit_logs`

append-only 테이블이다. `id` PK, `actor_kind(user/launcher/system)`, optional `actor_user_id`, `actor_access_code_id`, 당시 `actor_display_name`, `action VARCHAR(80)`, `resource_type VARCHAR(80)`, `resource_id VARCHAR(80) NULL`, `summary TEXT`, `request_id VARCHAR(64) NULL`, `metadata_json JSON NULL`, `created_at DATETIME`을 가진다. actor FK는 RESTRICT다.

로그에는 전화번호, 접속 코드 원문, cookie/token, 업로드 원문을 넣지 않는다.

### `operation_jobs`

| 컬럼 | 타입 | Null/기본값 | 제약과 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `job_type` | VARCHAR(24) | NOT NULL | `import`, `export`, `backup`, `restore`, `notification` |
| `status` | VARCHAR(16) | NOT NULL / `queued` | `queued`, `running`, `completed`, `failed`, `cancelled` |
| `source_name` | VARCHAR(255) | NULL | 사용자에게 보여줄 원본명 |
| `output_relative_path` | VARCHAR(260) | NULL | runtime 기준 경로 |
| `requested_by_user_id` | INTEGER | NULL | FK `users.id` RESTRICT, system 작업은 NULL |
| `requested_by_access_code_id` | INTEGER | NULL | FK `access_codes.id` RESTRICT |
| `total_count`, `success_count`, `failure_count` | INTEGER | NOT NULL / 0 | 모두 `>= 0`, success+failure <= total |
| `error_summary` | TEXT | NULL | 대표 오류 |
| `metadata_json` | JSON | NULL | 작업별 option/result |
| `created_at`, `started_at`, `completed_at` | DATETIME | lifecycle | UTC |

### `operation_job_errors`

`id` PK, `job_id FK operation_jobs.id CASCADE`, optional `row_number >= 1`, optional `field_name`, `message TEXT NOT NULL`, optional `raw_value TEXT`, `created_at DATETIME NOT NULL`을 가진다. 개인정보가 있는 전체 행을 raw value에 저장하지 않는다.

## 동시성과 재기동

### `idempotency_records`

`id` PK, `namespace VARCHAR(160)`, `key_hash VARCHAR(255)`, `request_hash VARCHAR(255)`, `status(processing/completed/failed)`, optional `response_status`, optional `response_json`, `created_at`, `updated_at`, `expires_at`을 가진다. `UNIQUE (namespace, key_hash)`와 `expires_at > created_at`을 강제한다. namespace는 인증 주체와 command 종류를 포함하되 접속 code/session 원문을 포함하지 않는다.

### `operation_locks`

`resource_type VARCHAR(80)`, `resource_id VARCHAR(80)` 복합 PK, `operation VARCHAR(80)`, `owner_token VARCHAR(64)`, `acquired_at`, `expires_at`을 가진다. `expires_at > acquired_at`. 추첨과 복구처럼 단일 실행만 허용하는 작업이 transaction 안에서 획득하고, 만료된 lock은 재기동 후 회수할 수 있다.

## FK 삭제 정책

| 관계 | 정책 | 이유 |
|---|---|---|
| building → floors | CASCADE | 참조 공간이 없을 때만 건물의 순수 하위 기준을 함께 제거 |
| floor → locations | RESTRICT | 공간을 무명/NULL 층으로 만들지 않음 |
| location/role → assignments | CASCADE | 다대다 관계 row는 소유 엔티티와 함께 제거 |
| offering → schedules | CASCADE | 아직 운영되지 않은 개설 삭제 시 구성 행 정리 |
| registration → status history | CASCADE | 신청을 normal UI에서 hard delete하지 않는 전제의 소유 이력 |
| lottery run → targets/results | CASCADE | run aggregate 내부 snapshot |
| attendance session → records | CASCADE | 출석 회차 aggregate 내부 기록 |
| operation job → errors | CASCADE | job aggregate 내부 오류 |
| 그 외 업무/감사 FK | RESTRICT | 과거 신청, 추첨, 출석, 행위자 의미 보존 |

기준 데이터는 삭제보다 비활성화를 우선한다. hard delete API는 참조가 없는 draft/미사용 기준 데이터에만 허용한다. 운영 이력, 신청, 추첨 run, audit log는 normal UI에서 hard delete하지 않는다.

## 필수 인덱스

SQLite는 FK index를 자동 생성하지 않으므로 다음을 명시한다.

- 사용자/인증: `users(role,is_active)`, `access_codes(user_id)`, `access_codes(expires_at,revoked_at,hidden_at)`, `user_sessions(user_id)`, `user_sessions(access_code_id)`, `user_sessions(expires_at,revoked_at)`
- 공간: `building_floors(building_id)`, `locations(building_floor_id,is_active)`, `location_role_assignments(role_id)`
- 강좌: `courses(category_id,is_active)`, `course_offerings(term_id,status)`, `course_offerings(course_id)`, `course_offerings(instructor_id)`, `course_schedules(offering_id)`, `course_schedules(weekday,time_slot_id)`, `course_schedules(location_id)`
- 회원/신청: `members(name)`, `members(phone)`, `members(is_active)`, `registrations(member_id)`, `registrations(offering_id,status)`, `registration_status_history(registration_id,changed_at)`
- 추첨/출석: `lottery_runs(term_id,status)`, `lottery_run_targets(lottery_run_id)`, `lottery_run_targets(offering_id)`, `lottery_results(lottery_run_target_id)`, `lottery_results(registration_id)`, `attendance_sessions(offering_id,session_date)`, `attendance_records(registration_id)`
- 운영: `audit_logs(created_at)`, `audit_logs(resource_type,resource_id)`, `audit_logs(actor_user_id)`, `operation_jobs(job_type,status)`, `operation_job_errors(job_id)`, `idempotency_records(expires_at)`, `operation_locks(expires_at)`

unique constraint와 PK가 이미 만드는 index는 중복 생성하지 않는다. 실제 query plan과 데이터 규모 테스트 전에는 추측성 index를 더 추가하지 않는다.

## DB 밖 application 규칙

다음 규칙은 여러 row나 현재 시각/권한을 함께 봐야 하므로 DB CHECK만으로 처리하지 않는다.

- open term과 open offering에서만 신청 생성/변경 허용
- 회차별 최대 신청 수, 동일 회원 시간 충돌과 중복 선택
- schedule의 공간이 활성 상태이고 course-eligible 역할을 하나 이상 보유하는지 확인
- offering을 open하기 전에 최소 한 schedule, 유효한 정원과 강사/공간 준비 상태 확인
- gender-split 강좌 신청/추첨 시 회원 성별 `unknown` 처리
- lottery result의 registration이 target offering에 속하는지 확인
- attendance record의 registration이 attendance session offering에 속하는지 확인
- 현재 registration 상태와 status history를 같은 transaction에서 변경
- access code/user/session의 활성 조건을 로그인 시 모두 재검증
- optimistic version 불일치 시 `409 Conflict`로 최신 데이터 재조회 유도
- audit log와 WebSocket resource event는 업무 transaction commit 이후 기록/발행 경계 준수

## 초기 migration 생성 순서

1. 기관/인증: `organization_settings`, `users`, `access_codes`, `user_sessions`
2. 공간: `buildings`, `building_floors`, `location_roles`, `locations`, assignments
3. 강좌 기준: categories, courses, instructors, terms, time slots
4. 강좌 운영: offerings, schedules
5. 회원/접수: gender codes, members, registrations, status history
6. 추첨/출석
7. audit/jobs/idempotency/locks
8. enum seed, partial unique index와 검색 index

재구현 시 실제 SQLite schema를 승인 스냅샷으로 고정한다. 계약 테스트는 FK, UNIQUE, CHECK, partial index, cascade/restrict와 query index를 구조 및 대표 위반 SQL 양쪽에서 검증한다. schema 변경 시 이 문서, SQLAlchemy model, Alembic revision과 snapshot diff를 함께 검토한다.
