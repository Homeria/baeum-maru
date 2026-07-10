# 데이터 모델

## 기준

- 실제 스키마의 기준 파일은 `internal/migration/sql/001_init.sql`이다.
- 현재 개발 단계에는 실사용 데이터가 없으므로, 비호환 스키마 변경은 기존 DB를 폐기하고 진행한다.
- 데이터 모델은 정규화하되, 현장 사용자가 입력하는 단위를 잃지 않는다.

## 운영 기준 데이터

| 영역 | 테이블 | 의미 |
|---|---|---|
| 회원 | `members`, `gender_codes` | 회원 기본 정보와 성별 코드 |
| 회차 | `terms` | 2026년 2학기 같은 운영 기간과 접수 상태/신청 한도 |
| 강좌 | `course_categories`, `courses`, `instructors` | 분야, 과목 원형, 강사 마스터 |
| 공간 | `buildings`, `building_floors`, `locations` | 건물, 층, 실제 사용하는 공간 |
| 공간 용도 | `location_roles`, `location_role_assignments` | 강의, 접수, 사무, 행사 등 다대다 역할 |
| 시간 | `time_slots` | 09:00~09:50 같은 재사용 시간대 |

## 강좌 운영

```text
terms ─┐
courses ─┼─ course_offerings ── course_schedules ── time_slots
instructors ─┘           │
locations ───────────────┘
```

- `courses`는 과목의 정체성이다. 예: 컴퓨터 초급, 한글교실 초급
- `course_offerings`는 특정 회차의 실제 개설 건이다. 정원, 접수 상태, 공간, 강사를 가진다.
- `course_schedules`는 한 개설 강좌에 여러 요일/시간대를 연결한다.
- `capacity_type`은 `fixed`, `open`, `gender_split`을 지원한다.

## 접수와 결과

```text
members ── registrations ── course_offerings
                  │
                  ├─ registration_status_history
                  └─ lottery_results ── lottery_runs ── lottery_run_targets
```

- `registrations`의 `(member_id, offering_id)`는 유일하다.
- 상태는 `applied`, `cancelled`, `selected`, `waitlisted`, `rejected`, `confirmed`이다.
- 상태 변경 원인과 행위자는 이력으로 남긴다.
- 추첨 실행, 대상, 결과를 분리해 재추첨과 결과 재현을 명확히 한다.

## 인증과 운영 이력

- `users`, `access_codes`: 영구/임시 사용자, 역할, 만료, 폐기 상태
- `attendance_sessions`, `attendance_records`: 강좌 개설별 출석 회차와 기록
- `audit_logs`: 사용자/접속 코드/시스템이 수행한 감사 이력
- `operation_jobs`, `operation_job_errors`: 가져오기, 내보내기, 백업, 복원 같은 장시간 작업 이력
- `settings`: 기관별 운영 설정

## 공간과 층의 미해결 결정

현재 `building_floors`는 기준 테이블이지만 `locations`는 `floor_label` 문자열을 저장한다. 이는 층 이름을 손쉽게 표시하는 장점이 있지만, 층 이름 변경과 참조 무결성에서 중복이 생길 수 있다.

프로토타입 전환 단계에서 다음 중 하나를 확정한다.

1. `locations.building_floor_id`를 FK로 두고 표시명은 join으로 읽는다.
2. 야외/건물 미지정 공간까지 단순하게 다뤄야 한다면 `floor_label`을 유지하되, 층 마스터는 입력 보조용이라는 정책을 명시한다.

현재 목표인 건물/층 재사용과 중복 입력 방지에는 1번이 더 적합하다.
