# 스키마 재설계 메모

이 문서는 `refactor/schema-redesign` 브랜치에서 진행한 DB 스키마 대개편의 기준을 기록한다.

목표는 실제 기관 시간표의 `영역 - 과목명 - 정원 - 요일 - 시간 - 강의실` 구조를 DBMS에 맞게 정규화하면서, 이후 장소 공지/행사 디스플레이/백업/감사 같은 운영 기능으로 확장 가능한 기반을 만드는 것이다.

## 강좌 운영

강좌 운영은 다음 경계로 나눈다.

- `course_categories`: 평생교육, 취미여가 같은 상위 영역
- `courses`: 한글교실, 컴퓨터, 스마트폰 같은 과목 원형
- `terms`: 2026년 2학기 같은 운영 단위
- `course_offerings`: 특정 학기에 실제 운영되는 강좌 개설 건
- `time_slots`: 09:00-09:50 같은 시간대 마스터
- `course_schedules`: 운영 강좌와 요일/시간대 연결

`course_offerings`는 정원 표현을 숫자 하나로 제한하지 않는다.

- `fixed`: 일반 숫자 정원
- `open`: 개방형 정원
- `gender_split`: 남녀 분리 정원

같은 과목이 같은 학기에 1반, 2반, 3반으로 열릴 수 있으므로 `UNIQUE(term_id, course_id)`를 제거한다. 한 운영 강좌가 월/수처럼 여러 요일을 가질 수 있으므로 시간표를 별도 연결 테이블로 둔다.

## 장소

장소는 물리 공간과 용도를 분리한다.

- `buildings`: 본관, 별관 같은 건물 마스터
- `locations`: 문화교육실, 컴퓨터실, 대강당 같은 물리 장소
- `location_roles`: 강의, 사무, 접수, 행사, 보관 같은 장소 역할
- `location_role_assignments`: 장소와 역할의 다대다 연결

기존 `classrooms` 테이블은 강의실만 표현하므로 `locations`에 흡수한다. 강좌는 `course_offerings.location_id`로 장소를 참조한다.

공지사항이나 행사 디스플레이는 장소 자체에 문구를 저장하지 않고, 별도 기능에서 `location_id`를 참조하도록 한다. 장소는 재사용 가능한 기준 데이터로 유지한다.

## 추첨

추첨은 실행, 대상, 결과를 분리한다.

- `lottery_runs`: 추첨 실행 단위
- `lottery_run_targets`: 실행 대상 강좌
- `lottery_results`: 신청 건별 추첨 결과

기존에는 결과 테이블을 통해 어떤 강좌의 추첨인지 추론했다. 이제 실행 대상은 `lottery_run_targets`에 명시한다. 이 구조는 신청자가 0명인 추첨, 재추첨, 실행 이력 조회에서 더 명확하다.

## 운영 이력

운영 이력은 웹 사용자, 접속 코드 사용자, 시스템 작업을 모두 추적할 수 있게 열어둔다.

- `users.user_kind`: 장기 계정과 접속 코드 기반 임시 계정을 구분한다.
- `registration_status_history.actor_kind`, `actor_display_name`, `metadata_json`: 사용자/접속 코드/시스템 작업을 같은 이력 테이블에서 추적한다.
- `audit_logs.actor_kind`, `actor_display_name`, `metadata_json`: 감사 로그에 행위자 유형과 구조화 메타데이터를 남길 수 있게 한다.
- `operation_jobs`: 가져오기, 내보내기, 백업, 복원, 알림 같은 작업 실행 이력
- `operation_job_errors`: 가져오기 실패 행처럼 작업별 상세 오류

가져오기/내보내기/백업/알림은 각자 별도 이력 테이블을 만들 수도 있지만, MVP 이후 기능 확장 단계에서는 공통 작업 테이블이 더 단순하다. 작업별 고유 정보는 `metadata_json`에 보관하고, 자주 조회해야 하는 항목이 생기면 그때 별도 컬럼으로 승격한다.

## 코드 반영

- `CourseOffering.Schedules`를 실제 `course_schedules`에서 로드한다.
- 신청 시간 충돌 검사는 대표 시간 하나가 아니라 전체 스케줄을 기준으로 검사한다.
- 추첨 저장 시 `lottery_run_targets`에 대상 강좌를 저장한다.
- 최근 추첨 조회는 `lottery_results` 추론이 아니라 `lottery_run_targets` 기준으로 조회한다.

## 후속 작업

- 강좌 관리 UI는 아직 기존 입력 흐름을 최대한 유지한다. 다음 UI 개편에서 다중 요일, 시간대 선택, 정원 타입 선택을 명확히 드러낸다.
- 런처 장소 UI는 이 스키마가 develop에 들어간 뒤 다시 merge해서 `buildings`, `locations`, `location_roles` 기준으로 재정렬한다.
- 신청, 추첨, 출석은 계속 `course_offerings.id`를 기준으로 동작한다.
