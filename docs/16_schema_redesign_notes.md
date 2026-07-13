# 스키마 재설계 결정 기록

## 상태

2026-07-13 `docs/python-schema-baseline`에서 결정 완료. 구현 기준은 `08_data_model.md`다. 이 문서는 과거 Go schema와 달라진 이유만 기록한다.

## 유지한 핵심

- 강좌는 `분야 → 과목 → 회차별 개설 → 복수 시간표`로 분리한다.
- 공간은 `건물 → 층 → 공간`과 `공간 ↔ 역할`을 분리한다.
- 신청은 회원과 강좌 개설의 관계이며 현재 상태와 상태 이력을 함께 가진다.
- 추첨은 실행, 대상 snapshot, 결과를 분리해 재실행과 감사를 보존한다.
- 사용자/접속 코드/감사 로그/작업 이력으로 운영 행위자를 추적한다.

## 과거 schema에서 변경한 사항

| 과거 구조 | Python 기준선 | 이유 |
|---|---|---|
| `locations.building_id + floor_label` | `locations.building_floor_id` | 건물/층 중복 제거와 참조 무결성 |
| 역할 이름으로 강의실 판단 | `location_roles.is_course_eligible` | 사용자 정의 역할명을 의미 코드로 사용하지 않음 |
| offering의 `display_name`, `level_label`, `section_label` | course name + optional `section_label` | 표시명/난이도 중복 제거 |
| offering이 공간 하나 참조 | schedule마다 `location_id` 참조 | 요일별 다른 공간 허용 |
| `registration_enabled`와 offering status 병존 | status만 사용 | 동일 의미의 상태 중복 제거 |
| password/username과 access user 혼합 | 감사용 `users` + `access_codes` | 프로토타입은 접속 코드 인증만 사용 |
| access code `status`에 expired 저장 | `expires_at`, `revoked_at`, `hidden_at`에서 계산 | 시간 경과에 따른 상태 불일치 제거 |
| 접속 코드 원문을 일반 API도 조회 가능 | hash + launcher 전용 `display_code` | 런처 재확인 UX를 유지하며 LAN 노출 차단 |
| 범용 `settings(key,value)` | singleton `organization_settings` | EAV를 피하고 설정 타입/제약 명시 |
| 프로세스 메모리 session/lock 가능성 | `user_sessions`, `idempotency_records`, `operation_locks` | 재기동 가능한 stateless server |
| 생년월일 저장 | 필수 schema에서 제거 | 현재 업무에 불필요한 개인정보 최소화 |

## 현장 시간표 대응

`평생교육 - 한글교실 초급 - 20명 - 월/수 - 14:00~14:50 - 문화교육실(3층)`은 다음으로 저장한다.

```text
course_categories: 평생교육
courses: 한글교실 초급
terms: 2026년 2학기
course_offerings: fixed 정원 20, 분반 없음, 개설 상태
course_schedules: 월 + 14:00~14:50 + 문화교육실
                  수 + 14:00~14:50 + 문화교육실
```

`컴퓨터초급 1반`과 `컴퓨터초급 2반`은 course를 중복 생성하지 않고 같은 course를 참조하는 offering 두 개와 각각의 `section_label`로 표현한다. 과목명 자체가 `한글교실 초급`인 경우 난이도는 course 이름의 일부이며 별도 level table을 만들지 않는다.

## 의도적 비정규화

- `registrations.status`와 `registration_status_history`는 현재 projection과 append-only 감사라는 책임이 다르다.
- `lottery_run_targets`의 정원/신청 수 snapshot은 실행 당시 결과 재현을 위해 원본 offering과 중복한다.
- `actor_display_name`은 사용자의 이름이 나중에 수정되어도 당시 감사 표시를 유지하기 위한 snapshot이다.
- `access_codes.display_code`와 `code_hash`는 런처 재확인 값과 인증 verifier라는 서로 다른 보안 책임을 가진다.

그 외 표시 가능한 값은 join으로 계산한다. course display name, 건물/층 표시, 접속 코드 만료 상태를 별도 컬럼으로 복제하지 않는다.

## UI와 정규화 경계

React 화면은 정규화된 테이블을 하나씩 노출하지 않는다. 사용자는 회차, 분야, 과목, 강사, 공간, 요일/시간대를 하나의 강좌 개설 폼에서 조합하고 API command가 필요한 관계를 transaction으로 저장한다.

건물/층/역할도 먼저 기준 데이터로 등록하지만 공간 폼에서는 활성 값만 선택한다. reference data의 분리와 사용자의 입력 단계 수는 API query와 복합 form으로 조정하며 schema를 다시 중복시키지 않는다.
