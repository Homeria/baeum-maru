# Python 프로토타입 브랜치 로드맵

## 목적

`Python + FastAPI + SQLite + React/Vite/TypeScript + pywebview` 기반 프로토타입을 완성하기 위한 작업 순서다. 다음 브랜치를 만들거나 구현하기 전에 이 문서를 단일 기준으로 확인한다.

## 진행 원칙

- 모든 작업 브랜치는 최신 `develop`에서 만들고 PR 검증 후 `develop`에 병합한다.
- `main`은 사용자가 명시적으로 요청하기 전까지 변경하거나 PR 대상으로 사용하지 않는다.
- Go 구현은 `go-prototype-baseline-2026-07` 태그에서만 보존하며 활성 코드에 호환 계층을 두지 않는다.
- 실사용 데이터가 없으므로 초기 Alembic 이력은 최신 스키마 하나에서 시작한다.
- 7번 CI 기반 전까지는 로컬 전체 검증과 PR 검토를 병합 조건으로 삼는다. 7번 이후에는 PR CI 통과도 필수다.
- `manual` 브랜치는 실제 Windows 또는 다중 브라우저 확인이 필요하므로 사용자 피드백 전에는 병합하지 않는다.
- 백엔드 요청 흐름은 `router → service → repository → database`로 읽혀야 한다.
- 업무 기능은 DB/service/API/UI/test를 함께 끝내는 세로 슬라이스로 구현한다.
- 순서를 바꾸면 이 문서에 dependency와 이유를 먼저 반영한다.

## 현재 위치

- Go checkpoint: `go-prototype-baseline-2026-07`
- 이전 Python feature-first/ORM 구현: 구조 재검토로 폐기
- 최근 완료: `feat/application-events-audit`
- 다음 예정: `feat/realtime-websocket-foundation`
- merge target: `develop` only

## A. 실행 및 아키텍처 기반

1. `refactor/readable-layered-boilerplate`: 기존 Python 구현 폐기, 역할 docstring만 가진 수평 계층 확정
2. `feat/config-runtime-foundation`: portable runtime path, JSON/env 설정과 structured logging
3. `feat/database-core`: SQLAlchemy Base, engine, Session factory와 SQLite PRAGMA
4. `feat/database-schema-baseline`: 최신 모델 전체와 단일 Alembic initial revision 재구현
5. `test/sqlite-schema-contract`: FK, unique, check, index, WAL, busy timeout 재검증
6. `feat/api-foundation`: FastAPI app, `/api/v1`, 공통 오류, request ID, pagination과 OpenAPI metadata
7. `ci/python-react-foundation`: backend/frontend 기본 PR quality gate
8. `feat/application-events-audit`: commit 이후 event와 audit sink 경계
9. `feat/realtime-websocket-foundation`: 인증 가능한 WebSocket, heartbeat, resource event와 재연결 계약
10. `feat/frontend-integration-foundation`: typed API client, TanStack Query, WebSocket invalidation과 launcher bridge type
11. `feat/fastapi-react-static-serving`: production asset와 history fallback 제공
12. `feat/launcher-control-bridge`: pywebview bridge allowlist와 lifecycle state machine
13. `feat/launcher-shell-foundation`: persistent tab 기반 독립 런처 shell
14. `feat/launcher-server-lifecycle`: 서버 시작/중지/재시작과 bind/port/address 상태
15. `feat/windows-pywebview-onedir`: WebView2 기반 PyInstaller `onedir` artifact
16. `test/backend-startup-contract`: 설정 오류, 경로와 lifespan 실패 동작 검증
17. `test/api-foundation-contract`: 오류 형식, pagination과 OpenAPI 기본 계약 검증
18. `test/launcher-bridge-security`: bridge allowlist와 원격 호출 차단 검증
19. `manual/windows-bootstrap-smoke`: Python 없는 Windows, 한글/공백 경로, WebView2와 시작 속도 확인

종료 기준: 공통 실행 경계, DB transaction, REST/WebSocket 계약, React와 pywebview 배포 경로가 실제 Windows에서 검증된다.

## B. 접근 및 기관 환경

20. `feat/identity-access-code`: access code, session, role, expiry/revoke 정책과 API
21. `feat/operator-auth-shell`: 로그인, 현재 session, 권한 메뉴, 만료 UX
22. `feat/operator-realtime-query-sync`: query cache 무효화, reconnect reconciliation, 편집 중 충돌 표시
23. `feat/launcher-access-management`: 코드 발급/연장/폐기/삭제와 접속 현황
24. `feat/organization-settings`: 기관 정보, 로고, 신청 가능 개수와 운영 설정
25. `feat/location-domain-api`: building, floor, space, role, 다중 역할의 application/API
26. `feat/operator-location-management`: 공간 기준정보 전체 관리 화면
27. `manual/location-management-ux`: 공간/건물/층/역할 등록과 한글 입력 검증

종료 기준: 호스트 담당자가 기관 환경과 접근 권한을 준비하고 직원이 안전하게 로그인한다.

## C. 강좌, 회원 및 접수

28. `feat/course-reference-domain-api`: term, category, instructor, time slot의 application/API
29. `feat/course-offering-domain-api`: course, offering, multiple schedule와 충돌 규칙
30. `feat/operator-course-management`: 강좌 기준정보와 개설 강좌 관리 화면
31. `manual/course-management-ux`: 실제 시간표 입력과 수정 흐름 검증
32. `feat/member-domain-api`: 회원 등록/수정/상태와 검색 query
33. `feat/operator-member-management`: 회원 목록, 상세, 수정, 신청 이력
34. `feat/registration-domain-api`: 신청, 중복/시간/개수 제한, 상태 이력
35. `feat/reception-submission-api`: 회원과 복수 강좌 신청의 원자적 command
36. `feat/operator-reception-workspace`: 한 화면 회원 검색/입력, 강좌 선택, 접수 결과
37. `manual/multi-user-reception`: 2~5개 브라우저 동시 접수와 즉시 갱신 검증

종료 기준: 여러 직원이 같은 데이터를 보며 회원과 복수 강좌 신청을 안전하게 접수한다.

## D. 추첨, 출석 및 운영 기능

38. `feat/lottery-domain-api`: seeded draw, result, waitlist, rerun guard와 잠금
39. `feat/operator-lottery`: 준비, 실행, 결과, 재추첨 확인 화면
40. `feat/attendance-domain-api`: 수업 회차와 출석 일괄 저장/query
41. `feat/operator-attendance`: 출석 입력과 이력 화면
42. `feat/excel-domain-api`: 회원/강좌 import 검증과 업무별 export job
43. `feat/operator-excel-operations`: 업로드 오류 확인, 작업 상태, 다운로드
44. `feat/backup-restore-domain-api`: backup 생성/목록과 restart-required restore queue
45. `feat/launcher-backup-logs`: backup/restore, 분류 로그, 오류 상세
46. `feat/audit-operation-jobs`: 감사 로그와 장기 작업 상태/오류
47. `feat/operator-operations`: audit, job, 운영 설정 화면
48. `test/core-workflow-characterization`: 접수→추첨→확정→출석→Excel→backup/restore 통합 검증
49. `feat/launcher-dashboard-settings`: 운영 요약, 네트워크, 서버 설정과 기관 현황
50. `manual/launcher-operation-ux`: 런처 전체 lifecycle과 장애 표시 검증

종료 기준: 교육 담당자가 한 운영 회차의 핵심 업무와 백업을 끝까지 수행한다.

## E. 보안, 동시성 및 배포 완성

51. `feat/security-csrf-headers`: CSRF, CSP, security header, upload boundary
52. `feat/security-login-throttling`: 로그인 실패 제한, lockout, audit
53. `feat/security-local-https`: 로컬 인증서 생성, 갱신, trust 안내와 Secure cookie
54. `feat/concurrency-idempotency-locks`: 중복 submit, optimistic conflict, lottery/restore lock
55. `feat/access-session-presence`: active session과 last-seen 현황
56. `test/multi-client-concurrency`: 동시 접수, WebSocket 단절/재연결, stale UI 검증
57. `test/large-dataset-performance`: 회원 1,000명, 강좌 50개, 신청 3,000건
58. `test/failure-backup-recovery`: 강제 종료, 네트워크 단절, backup/restore 실패 복구
59. `test/openapi-contract`: endpoint, DTO, error, permission schema snapshot
60. `ci/frontend-windows-package`: React, Python, OpenAPI client, PyInstaller Windows gate
61. `feat/windows-portable-package`: versioned ZIP, runtime tree, first-run guide, checksum
62. `manual/windows-portable-smoke`: 일반 사무용 Windows에서 full package 검증
63. `docs/operator-guide`: 설치, 런처, 접수, 추첨, 출석, backup 안내
64. `docs/troubleshooting-security`: 방화벽, 인증서, 백신, 복구, 로그 안내
65. `manual/prototype-operation-simulation`: 실제 시간표 기반 한 회차 운영 rehearsal

종료 기준: 보안, 성능, 동시성, 복구 및 배포 점검을 통과한 Python 프로토타입을 `develop`에 남긴다. `main` 병합과 release tag는 사용자가 별도로 요청할 때만 진행한다.
