# Python 초기 스키마 기준선

## 상태와 범위

이 문서는 Python 프로토타입의 단일 초기 SQLite 스키마 설계 명세다. `app/db/schema/`의 도메인별 Python DDL이 이 기준을 구현하며 이후 변경도 이 문서와 함께 갱신한다.

- 과거 Go DB와의 호환 migration은 작성하지 않는다.
- 실사용 데이터가 생기기 전에는 DB 파일을 재생성하고, 이후에는 별도 migration 체계를 도입한다.
- 현재 범위는 기관 한 곳과 SQLite 파일 하나다. 모든 테이블에 tenant/organization FK를 반복하지 않는다.
- 현재 **22개 테이블 / 8개 도메인 모듈**로 구성한다.

## 확정한 모델링 결정

1. **공간은 `buildings` → `building_floors` → `spaces` 3단 계층**이다. 장소 등록 시 건물·층을 드롭다운으로 고르고 이름만 입력한다(비전문가 입력 최소화). 과거 `locations` 명칭은 `spaces`로 바꾼다.
2. 층은 숫자가 아니라 `building_floors.label` 문자열이다. `지하 1층`, `옥상` 등을 지원하며 건물별로 목록을 등록한다.
3. **장소 유형은 `space_types` 마스터**(관리자 정의, 드롭다운)로 두고, 강좌 배정 가능 여부는 `space_types.is_course_eligible`로 판단한다. 과거의 역할 M:N(`location_roles`/`location_role_assignments`)은 폐기했다.
4. **강좌 정체성 = 영역(`course_categories`) + 과목명(`courses.name`) + 난도(`course_levels`)**. 난도는 마스터에서 고르며(초급/중급 등, 기관 정의), 난도가 다르면 별개의 `courses` 행이다.
5. **반(분반)은 `course_offerings.section_label`**로 표현한다(예: 1반/2반, 자동 넘버링). 표시 과목명은 `name + [난도] + [반]`을 앱에서 조합한다.
6. 공간은 `course_offerings`가 아니라 각 `course_schedules`가 참조한다. 같은 개설 강좌가 요일별로 다른 공간을 쓸 수 있다.
7. 직원 로그인은 접속 코드 방식만 사용한다. **`operators`**(과거 `users`)는 행위자 프로필이며 username/password를 저장하지 않는다.
8. 접속 코드와 세션의 만료 상태는 `expires_at`에서 계산한다. 만료 상태를 별도 저장하지 않는다.
9. **접속 코드는 `code_hash`(HMAC + 서버 pepper)만 저장한다.** 원문(`display_code`)은 저장하지 않고 발급 시 1회 표시하며, 잊으면 재발급한다(코드는 짧은 만료 + 즉시 폐기 전제).
10. 신청의 현재 상태(`registrations`)와 상태 이력(`registration_status_history`)은 의도적으로 함께 저장한다.
11. **대기 순번은 두 곳**: `registrations.waitlist_order`(현재 살아있는 대기열, 승계로 변함) + `lottery_results.result_order`(추첨 시점 스냅샷, 불변). 후자로 전자를 초기화한다.
12. 추첨 대상의 정원과 인원은 실행 당시 snapshot으로 남긴다(재현·감사용). `gender_split`은 `eligible_male`/`eligible_female`까지 남긴다.
13. **추첨 결과는 백엔드가 seed로 확정**하고 프론트는 결과를 향해 애니메이션만 한다([문서 20](20_lottery_draw_flow.md)).
14. **기관 설정(기관명·로고·운영 기본값)은 DB가 아니라 설정 파일**(`core/settings.py` 계층 로더)로 둔다. 과거 `organization_settings` 테이블은 폐기했다.
15. **`version`(낙관적 잠금)은 두지 않는다.** 수정은 단일 노트북에서만 일어나 동시 수정 충돌이 없다.
16. **출석 기능은 두지 않는다.** "필요할 때만 켜는 런처" 배포 모델은 매 수업 상시 가동이 필요한 출석과 맞지 않는다. 상시 서버로 전환하면 독립 도메인으로 추가한다.
17. **동시성 조정(단일 실행 잠금·요청 멱등성)은 DB 테이블이 아니라 인메모리로 둔다.** 서버가 단일 프로세스라 공유 저장소 조정이 불필요하다. 중복은 도메인 UNIQUE 제약으로도 막는다. 과거의 `idempotency_records`/`operation_locks`는 폐기했다(다중 프로세스 확장 시 재도입).

## 공통 타입과 규칙

| 논리 타입 | SQLite 저장 | 규칙 |
|---|---|---|
| PK | `INTEGER` | 업무 테이블은 자동 증가 정수 ID (세션 등 일부는 자연키) |
| 문자열 | `TEXT` | API에서 trim/길이 검증 |
| bool | `INTEGER` | `0/1` CHECK |
| 날짜/시간/시각 | ISO `TEXT` | 앱이 **ISO 문자열로 일관 주입**(`YYYY-MM-DD`, `HH:MM[:SS]`, UTC datetime). CHECK 비교가 사전식이므로 포맷 혼재 금지 |
| JSON | `TEXT` | object만 허용, Pydantic 검증 |

변경 가능한 엔티티는 `created_at`, `updated_at`을 가진다. `version`은 두지 않는다(단일 머신). append-only 이력과 worker 상태에는 timestamp만 둔다.

이름과 회원번호는 trim한 값을 저장한다. 전화번호는 숫자만 저장하고 화면에서 포맷한다. 주민번호·생년월일은 프로토타입에 저장하지 않는다.

## 테이블 목록

| 영역 | 테이블 |
|---|---|
| 인증(identity) | `operators`, `access_codes`, `operator_sessions` |
| 공간(spaces) | `buildings`, `building_floors`, `space_types`, `spaces` |
| 강좌 기준 | `course_categories`, `course_levels`, `courses`, `instructors`, `terms`, `time_slots` |
| 강좌 운영 | `course_offerings`, `course_schedules` |
| 회원/접수 | `members`, `registrations`, `registration_status_history` |
| 추첨 | `lottery_runs`, `lottery_run_targets`, `lottery_results` |
| 운영 이력 | `audit_logs` |

> 기관 설정은 테이블이 아니라 설정 파일에 있다(결정 14). 성별은 `members.gender` CHECK(`male`/`female`)로 두고 별도 코드 테이블을 두지 않는다.

## 인증 (identity)

### `operators`

접속 코드를 발급받는 직원/자원봉사자의 행위자 프로필이다. 영구 비밀번호 계정이 아니다.

| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `display_name` | TEXT | NOT NULL | 행위자 표시명 |
| `role` | TEXT | NOT NULL | `staff`, `temporary_staff`, `viewer` |
| `is_active` | INTEGER | NOT NULL / 1 | 비활성 사용자는 로그인 불가 |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

이름은 unique가 아니다(동명이인 허용). 감사 FK가 있으면 hard delete하지 않고 비활성화한다.

### `access_codes`

| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `operator_id` | INTEGER | NOT NULL | FK `operators.id` RESTRICT |
| `code_hash` | TEXT | NOT NULL | UNIQUE, HMAC(코드, 서버 pepper) — 조회+검증 겸용 |
| `issued_at` | TEXT | NOT NULL | ISO |
| `expires_at` | TEXT | NOT NULL | `expires_at > issued_at` |
| `revoked_at` | TEXT | NULL | NULL이면 미폐기 |
| `last_used_at` | TEXT | NULL | 마지막 성공 인증 시각 |

코드 원문은 저장하지 않는다(결정 9). 표시 상태(사용 가능/만료/폐기)는 `expires_at`·`revoked_at`·operator 활성 여부로 계산한다.

### `operator_sessions`

| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK(대리키) |
| `operator_id` | INTEGER | NOT NULL | FK `operators.id` RESTRICT |
| `access_code_id` | INTEGER | NOT NULL | FK `access_codes.id` RESTRICT |
| `token_hash` | TEXT | NOT NULL | UNIQUE, 세션 토큰 해시(쿠키엔 원본) |
| `issued_at` | TEXT | NOT NULL | ISO |
| `expires_at` | TEXT | NOT NULL | `expires_at > issued_at` |
| `last_seen_at` | TEXT | NOT NULL | `>= issued_at`, presence |
| `revoked_at` | TEXT | NULL | logout/강제 종료 |

서버 세션(불투명 토큰)을 쓴다. 프로세스 재시작 후 DB만으로 인증 상태를 판정하고, 폐기가 필요하므로 JWT 대신 이 방식을 쓴다.

## 공간 (spaces)

### `buildings`
`id` PK, `name TEXT NOT NULL UNIQUE`, `description TEXT`, `sort_order`, `is_active`, `created_at/updated_at`.

### `building_floors`
`id` PK, `building_id` FK `buildings.id` **CASCADE**, `label TEXT NOT NULL`(숫자 제한 없음), `sort_order`, `is_active`, `created_at/updated_at`, `UNIQUE (building_id, label)`. 건물별 층 목록을 등록해 장소 등록 시 드롭다운으로 쓴다.

### `space_types`
`id` PK, `name TEXT NOT NULL UNIQUE`, `is_course_eligible INTEGER NOT NULL DEFAULT 1`, `sort_order`, `is_active`, `created_at/updated_at`. 관리자가 채우는 장소 유형 마스터(강의실/사무실 등). 초기 seed로 강의실/사무실/다목적실/기타를 제공한다.

### `spaces`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `building_floor_id` | INTEGER | NOT NULL | FK `building_floors.id` RESTRICT |
| `space_type_id` | INTEGER | NOT NULL | FK `space_types.id` RESTRICT |
| `name` | TEXT | NOT NULL | 예: 문화교육실, 대강당 |
| `sort_order`, `is_active` | INTEGER | | 목록 순서/활성 |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

`UNIQUE (building_floor_id, name)`. 정원은 장소가 아니라 강좌(`course_offerings`)에 둔다.

## 강좌 기준과 개설 (courses)

### `course_categories`
`id` PK, `name TEXT NOT NULL UNIQUE`, `sort_order`, `is_active`, `created_at/updated_at`. 예: 평생교육, 취미여가.

### `course_levels`
`id` PK, `name TEXT NOT NULL UNIQUE`, `sort_order`, `is_active`, `created_at/updated_at`. 난도 마스터(관리자 정의, seed 없음). 초급/중급 등.

### `courses`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `category_id` | INTEGER | NOT NULL | FK `course_categories.id` RESTRICT |
| `level_id` | INTEGER | NULL | FK `course_levels.id` RESTRICT (난도 없음 = NULL) |
| `name` | TEXT | NOT NULL | 기본 과목명(난도·반 제외). 예: 한글교실 |
| `description` | TEXT | NULL | 회원 노출 설명 |
| `is_active` | INTEGER | NOT NULL / 1 | |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

`level_id`가 nullable이라 부분 유니크 인덱스 2개로 고유성을 강제한다.
- `UNIQUE (category_id, name) WHERE level_id IS NULL`
- `UNIQUE (category_id, name, level_id) WHERE level_id IS NOT NULL`

### `instructors`
`id` PK, `name TEXT NOT NULL`, `phone TEXT NULL`, `is_active`, `created_at/updated_at`. 동명이인 허용(이름 unique 아님).

### `terms`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `name` | TEXT | NOT NULL | UNIQUE, 예: 2026년 2학기 |
| `starts_on`, `ends_on` | TEXT | NULL | 둘 다 있으면 `starts_on <= ends_on` |
| `registration_opens_at`, `registration_closes_at` | TEXT | NULL | 둘 다 있으면 opens < closes |
| `max_registrations_per_member` | INTEGER | NOT NULL / 0 | `>= 0`, 0은 제한 없음 |
| `status` | TEXT | NOT NULL / `draft` | `draft`, `open`, `closed`, `finalized` |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

### `time_slots`
`id` PK, `name TEXT NOT NULL UNIQUE`(교시 라벨), `start_time TEXT NOT NULL`, `end_time TEXT NOT NULL`, `sort_order`, `is_active`, `created_at/updated_at`, `CHECK (start_time < end_time)`, `UNIQUE (start_time, end_time)`.

### `course_offerings`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `term_id` | INTEGER | NOT NULL | FK `terms.id` RESTRICT |
| `course_id` | INTEGER | NOT NULL | FK `courses.id` RESTRICT |
| `section_label` | TEXT | NULL | 반. 예: 1반, 2반 (빈 문자열 금지) |
| `instructor_id` | INTEGER | NULL | FK `instructors.id` RESTRICT |
| `capacity_type` | TEXT | NOT NULL / `fixed` | `fixed`, `open`, `gender_split` |
| `capacity_total` | INTEGER | NULL | fixed일 때 `> 0` |
| `male_capacity`, `female_capacity` | INTEGER | NULL | gender_split일 때 각각 `>= 0`, 합 `> 0` |
| `status` | TEXT | NOT NULL / `draft` | `draft`, `open`, `closed`, `cancelled` |
| `sort_order` | INTEGER | NOT NULL / 0 | |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

정원 CHECK: fixed(총원만) / open(모두 NULL) / gender_split(남·여만, 합>0). 분반 고유성은 부분 유니크 2개(`WHERE section_label IS NULL` / `IS NOT NULL`).

### `course_schedules`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `offering_id` | INTEGER | NOT NULL | FK `course_offerings.id` CASCADE |
| `weekday` | INTEGER | NOT NULL | 1(월)..7(일) |
| `time_slot_id` | INTEGER | NOT NULL | FK `time_slots.id` RESTRICT |
| `space_id` | INTEGER | NOT NULL | FK `spaces.id` RESTRICT |

`UNIQUE (offering_id, weekday, time_slot_id)`. [월,수]처럼 복수 요일은 여러 행으로 표현한다.

## 회원과 접수

### `members`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK(대리키) |
| `member_no` | TEXT | NOT NULL | UNIQUE, 기관 회원번호(업무키, 수동 입력) |
| `name` | TEXT | NOT NULL | 검색 대상 |
| `gender` | TEXT | NOT NULL | `CHECK (gender IN ('male','female'))` |
| `phone` | TEXT | NOT NULL | 숫자만 저장 |
| `is_active` | INTEGER | NOT NULL / 1 | 이용중지 회원 신규 신청 금지 |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

`member_no`는 연도+일련번호 등 의미를 담은 업무 식별자라 사람이 부여한다(자동 채번 아님). 이름·전화 중복은 허용한다.

### `registrations`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `member_id` | INTEGER | NOT NULL | FK `members.id` RESTRICT |
| `offering_id` | INTEGER | NOT NULL | FK `course_offerings.id` RESTRICT |
| `status` | TEXT | NOT NULL / `applied` | `applied`, `selected`, `waitlisted`, `rejected`, `confirmed`, `cancelled` |
| `waitlist_order` | INTEGER | NULL | 대기 순번(`status='waitlisted'`일 때만 의미) |
| `created_at`, `updated_at` | TEXT | NOT NULL | ISO |

`UNIQUE (member_id, offering_id)`. 취소 후 재신청은 같은 row를 다시 전이한다. 활성 대기자 순번 중복은 부분 유니크로 막는다: `UNIQUE (offering_id, waitlist_order) WHERE status='waitlisted' AND waitlist_order IS NOT NULL`. 당첨자 중도 포기 시 대기 최소 순번을 승격한다(앱 로직).

### `registration_status_history`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `registration_id` | INTEGER | NOT NULL | FK `registrations.id` CASCADE |
| `from_status` | TEXT | NULL | 최초 생성만 NULL |
| `to_status` | TEXT | NOT NULL | registration status enum |
| `reason` | TEXT | NULL | 전이 사유(자유 텍스트) |
| `actor_kind` | TEXT | NOT NULL | `operator`, `launcher`, `system` |
| `actor_operator_id` | INTEGER | NULL | FK `operators.id` RESTRICT |
| `actor_access_code_id` | INTEGER | NULL | FK `access_codes.id` RESTRICT |
| `actor_display_name` | TEXT | NULL | 당시 표시명 snapshot |
| `metadata_json` | TEXT | NULL | 부가 정보 |
| `changed_at` | TEXT | NOT NULL | ISO |

신청 생성/상태 변경 transaction은 현재 row와 history를 함께 기록한다.

## 추첨 (lottery)

추첨 실행 흐름(백엔드 seed·미리보기·확정)은 [문서 20](20_lottery_draw_flow.md) 참조. 미저장 미리보기를 채택하므로 저장된 run은 항상 완료된 추첨이며, 실행 상태(prepared/running)나 lifecycle 시각은 두지 않는다.

### `lottery_runs`
`id` PK, `term_id` FK `terms.id` RESTRICT, `seed INTEGER NOT NULL`(백엔드 생성), `executed_by_operator_id` FK `operators.id` RESTRICT, `created_at`(추첨 시각).

### `lottery_run_targets`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `lottery_run_id` | INTEGER | NOT NULL | FK `lottery_runs.id` CASCADE |
| `offering_id` | INTEGER | NOT NULL | FK `course_offerings.id` RESTRICT |
| `capacity_type` | TEXT | NOT NULL | 실행 당시 snapshot |
| `capacity_total`, `male_capacity`, `female_capacity` | INTEGER | NULL | snapshot |
| `eligible_count` | INTEGER | NOT NULL | `>= 0`, 실행 당시 신청 수 |
| `eligible_male`, `eligible_female` | INTEGER | NULL | gender_split 재현용, 각 `>= 0` |

`UNIQUE (lottery_run_id, offering_id)`. 정원 CHECK는 `course_offerings`와 동일.

### `lottery_results`
| 컬럼 | 타입 | Null/기본값 | 의미 |
|---|---|---|---|
| `id` | INTEGER | NOT NULL | PK |
| `lottery_run_target_id` | INTEGER | NOT NULL | FK `lottery_run_targets.id` CASCADE |
| `registration_id` | INTEGER | NOT NULL | FK `registrations.id` RESTRICT |
| `result` | TEXT | NOT NULL | `selected`, `waitlisted`, `rejected` |
| `result_order` | INTEGER | NOT NULL | `>= 1`, 결과 그룹 내 순번 → `registrations.waitlist_order` 초기화 |
| `created_at` | TEXT | NOT NULL | ISO |

`UNIQUE (lottery_run_target_id, registration_id)`, `UNIQUE (lottery_run_target_id, result, result_order)`.

## 운영 (operations)

### `audit_logs`
append-only. `id` PK, `actor_kind`(operator/launcher/system), `actor_operator_id` FK `operators.id` RESTRICT, `actor_access_code_id` FK `access_codes.id` RESTRICT, `actor_display_name`, `action`, `resource_type`, `resource_id`, `summary`, `request_id`, `metadata_json`, `created_at`. 전화번호·코드 원문·토큰·업로드 원문은 넣지 않는다. 감사는 성공한 쓰기와 같은 transaction에 기록하고, 실패·읽기·보안 이벤트는 별도 transaction으로 남긴다([문서 19](19_repository_transaction_and_audit.md)).

단일 테이블 + 구분 컬럼(`resource_type`/`actor_kind`) + 인덱스 + `metadata_json` 방식이다. 소규모·균일 모양이라 종류별 테이블 분리는 하지 않고, 특정 종류 조회가 잦아지면 부분 인덱스로 대응한다.

> **동시성 조정은 DB 테이블로 두지 않는다.** 서버가 단일 프로세스라 단일 실행 잠금(추첨·복원 등)은 인메모리 잠금으로, 중복 요청 방지는 도메인 UNIQUE 제약 + 인메모리 가드로 처리한다. 과거 `idempotency_records`/`operation_locks`는 폐기했다.
>
> **엑셀 import/export·백업/복원 작업 기록(`operation_jobs`/`operation_job_errors`)도 두지 않는다.** 해당 기능을 실제 구현할 때, 행별 오류 영속 저장이 필요하면 그 시점에 추가한다.

## FK 삭제 정책

| 관계 | 정책 |
|---|---|
| building → building_floors | CASCADE |
| building_floor → spaces | RESTRICT |
| space_type → spaces | RESTRICT |
| offering → schedules | CASCADE |
| registration → status history | CASCADE |
| lottery run → targets/results | CASCADE |
| 그 외 업무/감사 FK | RESTRICT |

기준 데이터는 삭제보다 비활성화를 우선한다. 운영 이력·신청·추첨 run·audit log는 일반 UI에서 hard delete하지 않는다.

## 필수 인덱스

SQLite는 FK index를 자동 생성하지 않으므로 명시한다.

- 인증: `operators(role,is_active)`, `access_codes(operator_id)`, `access_codes(expires_at,revoked_at)`, `operator_sessions(operator_id)`, `operator_sessions(access_code_id)`, `operator_sessions(expires_at,revoked_at)`
- 공간: `building_floors(building_id)`, `spaces(building_floor_id,is_active)`, `spaces(space_type_id)`
- 강좌: `courses(category_id,is_active)`, `courses(level_id)`, 부분유니크 2개, `course_offerings(term_id,status)`, `course_offerings(course_id)`, `course_offerings(instructor_id)`, 분반 부분유니크 2개, `course_schedules(offering_id)`, `course_schedules(weekday,time_slot_id)`, `course_schedules(space_id)`
- 회원/신청: `members(name)`, `members(phone)`, `members(is_active)`, `registrations(member_id)`, `registrations(offering_id,status)`, 대기순번 부분유니크, `registration_status_history(registration_id,changed_at)`
- 추첨: `lottery_runs(term_id)`, `lottery_run_targets(lottery_run_id)`, `lottery_run_targets(offering_id)`, `lottery_results(lottery_run_target_id)`, `lottery_results(registration_id)`
- 운영: `audit_logs(created_at)`, `audit_logs(resource_type,resource_id)`, `audit_logs(actor_operator_id)`

## DB 밖 application 규칙

- open term과 open offering에서만 신청 생성/변경 허용
- 회차별 최대 신청 수, 동일 회원 시간 충돌·중복 선택 검사
- schedule의 공간이 활성이고 course-eligible 유형인지 확인
- gender-split 신청/추첨 시 성별 매칭 처리
- 추첨은 백엔드 seed로 확정, 저장(commit)은 원자적 transaction(추첨결과 + `registrations.status`/`waitlist_order` + 감사)
- lottery result의 registration이 target offering에 속하는지 확인
- 대기 승계: 당첨 포기 시 대기 최소 순번을 승격
- 현재 registration 상태와 status history를 같은 transaction에서 변경
- audit log는 업무 변경과 같은 transaction에 기록, WebSocket resource event는 commit 이후 발행

## 초기 생성 순서 (모듈)

1. 인증: operators, access_codes, operator_sessions
2. 공간: buildings, building_floors, space_types, spaces
3. 강좌: categories, levels, courses, instructors, terms, time_slots, offerings, schedules
4. 회원/접수: members, registrations, status history
5. 추첨: runs, targets, results
6. 운영: audit_logs
7. seed(space_types)와 partial/검색 index

schema 변경 시 이 문서와 Python DDL을 함께 검토한다.
