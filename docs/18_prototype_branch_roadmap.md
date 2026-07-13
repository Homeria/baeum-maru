# Python 프로토타입 브랜치 로드맵

## 목적

Go 기능 검증 구현을 폐기하고 `Python + FastAPI + SQLite + React/Vite/TypeScript` 기반 프로토타입을 완성하기 위한 branch 순서다. 다음 branch를 만들거나 구현하기 전에 이 문서를 먼저 확인한다.

## 운영 규칙

- 모든 작업 branch는 최신 `develop`에서 만들고 PR과 CI 통과 후 `develop`에 merge한다.
- `main`은 사용자가 명시적으로 요청하기 전까지 변경하거나 PR 대상으로 사용하지 않는다.
- Go 구현은 `go-prototype-baseline-2026-07` tag에서만 보존하고 active tree에 fallback으로 남기지 않는다.
- 실사용 데이터가 없으므로 Go DB 호환 migration을 작성하지 않는다.
- 수동 확인 branch는 사용자의 실제 확인과 feedback 전에는 merge하지 않는다.
- 순서를 바꿔야 하면 먼저 roadmap 문서에서 이유와 dependency를 갱신한다.
- 각 branch는 test, documentation, CI 영향을 함께 완료한다.

## 현재 위치

- Go checkpoint: `go-prototype-baseline-2026-07` → `547977b`
- 최근 완료: `docs/pywebview-websocket-stack`
- 다음 예정: `refactor/python-project-reset`
- merge target: `develop` only

## 1. Python reset과 실행 기반

1. `docs/python-transition-roadmap`: Python 기술 결정, architecture, 전체 roadmap 확정
2. `refactor/python-project-reset`: Go/Fyne/template/Go CI 제거, FastAPI health, pyproject, pytest/Ruff/mypy CI 생성
3. `feat/python-config-runtime`: portable path, JSON/env 설정, runtime directory, structured logging
4. `docs/python-schema-baseline`: 최신 table/constraint/cascade와 building-floor-location FK 결정 확정
5. `feat/sqlalchemy-alembic-baseline`: SQLAlchemy base/session과 단일 최신 Alembic initial schema
6. `test/sqlite-schema-contract`: FK, unique, check, index, cascade/null, WAL/busy timeout 검증
7. `feat/react-workspace-foundation`: pnpm workspace, operator/launcher app, shared package, frontend CI
8. `feat/fastapi-react-static-serving`: production asset manifest와 FastAPI static/history fallback
9. `feat/windows-pywebview-onedir`: pywebview/WebView2 launcher shell과 Python/frontend asset을 포함한 portable directory build
10. `test/windows-python-bootstrap-smoke`: Python 미설치 Windows, WebView2 유무, 한글/공백 경로, startup/size/memory 확인 - 수동 확인

종료 기준: Go code 없이 Python health server와 두 React shell이 build되고 Windows portable artifact가 실행된다.

## 2. application과 업무 모듈

11. `refactor/python-application-boundaries`: api/application/domain/port/infrastructure 규칙과 composition root
12. `feat/sqlalchemy-unit-of-work`: request/task session과 multi-repository transaction
13. `feat/application-events-audit`: domain event, audit sink, commit 이후 WebSocket publisher 경계
14. `feat/identity-module`: user, access code, session, role, expiry/revoke policy
15. `feat/member-module`: member entity, repository, search, create/update rule
16. `feat/location-module`: building, floor, space, role, multi-role assignment
17. `feat/course-module`: term, category, course, instructor, offering, multi-schedule
18. `feat/registration-module`: application, duplicate/time/limit rule, status history
19. `feat/reception-submission-usecase`: member create/update와 multiple registration atomic command
20. `feat/lottery-module`: run, target, seeded result, rerun guard, waitlist promotion
21. `feat/attendance-module`: session과 attendance record
22. `feat/excel-operation-module`: member/course import와 업무별 export
23. `feat/backup-restore-module`: backup list/create, restore queue, restart application
24. `feat/settings-operation-jobs`: setting, long-running job, job error/status
25. `test/python-workflow-characterization`: 접수→추첨→확정→출석→Excel→backup/restore 전체 workflow

종료 기준: HTTP/UI 없이 Python application service와 실제 SQLite integration test만으로 한 업무 cycle이 동작한다.

## 3. FastAPI REST API와 실시간 계약

26. `feat/api-foundation`: `/api/v1`, health/readiness, common error, request ID, OpenAPI metadata
27. `feat/api-auth-sessions`: access code login/logout/current session/permission
28. `feat/api-members`: member search, detail, create, update
29. `feat/api-locations`: building, floor, space, role CRUD/status API
30. `feat/api-course-reference-data`: term, category, course, instructor, time slot API
31. `feat/api-course-offerings`: offering과 multiple schedule command/query
32. `feat/api-reception-submissions`: member와 multiple application atomic endpoint
33. `feat/api-registrations`: application search, cancel, confirm, status history
34. `feat/api-lottery-runs`: preview, execute, rerun, result
35. `feat/api-attendance`: session과 attendance bulk save/query
36. `feat/api-excel-operations`: upload validation, import/export job과 download
37. `feat/api-backup-restore`: backup status/create/list와 restore queue
38. `feat/api-settings-audit-jobs`: setting, audit log, operation job endpoint
39. `feat/api-realtime-events`: authenticated `/api/v1/events/ws`, resource event, heartbeat, reconnect reconciliation
40. `test/openapi-contract`: endpoint, DTO, error, permission, schema snapshot
41. `ci/python-api-contract`: Python quality gate와 OpenAPI/client diff PR 검사

종료 기준: React가 필요한 모든 업무를 `/api/v1`로 수행하고 OpenAPI와 실제 response가 일치한다.

## 4. React 직원 업무 앱

42. `feat/operator-design-system`: token, typography, icon button, form, table, dialog, notification
43. `feat/operator-auth-shell`: login, routing, permission menu, error boundary, session expiry
44. `feat/operator-realtime-query-sync`: TanStack Query와 WebSocket resource invalidation/reconnect
45. `feat/operator-reception-member-flow`: member search/select/new/edit
46. `feat/operator-reception-course-picker`: filter 가능한 multiple course selection과 conflict hint
47. `feat/operator-reception-submit`: validation, atomic submit, conflict/retry UX - 수동 확인
48. `feat/operator-member-management`: member list/detail/edit/application history
49. `feat/operator-location-management`: building, floor, space, role management
50. `feat/operator-course-operations`: term/reference/offering/multiple schedule workflow - 수동 확인
51. `feat/operator-registration-dashboard`: search/filter/status/cancel/confirm
52. `feat/operator-lottery`: preparation, execute, result, rerun confirmation
53. `feat/operator-attendance`: session, bulk attendance, history
54. `feat/operator-operations`: Excel, backup, setting, audit/job status
55. `test/operator-e2e-accessibility`: Playwright, keyboard, responsive, Korean IME, reconnect

종료 기준: 직원이 browser에서 한 회차의 핵심 업무를 수행하고 다른 사용자 변경이 안전하게 반영된다.

## 5. pywebview 호스트 런처

56. `feat/launcher-control-bridge`: typed pywebview bridge, lifecycle state machine, child process control, network resolver
57. `feat/launcher-shell-foundation`: pywebview React app, persistent tabs, status/error shell, local asset policy
58. `feat/host-dashboard-server-control`: start/stop/restart, bind mode, port, access URL - 수동 확인
59. `feat/host-access-management`: access code issue/extend/revoke/delete/presence summary
60. `feat/host-environment-setup`: organization, logo, building/floor/space/role, default limits
61. `feat/host-logs-backup-settings`: categorized log, backup/restore, network/config, lab placeholders - 수동 확인
62. `test/launcher-control-isolation`: remote navigation/bridge 차단, command race, child crash, graceful shutdown

종료 기준: 호스트 담당자가 pywebview 독립 런처만으로 업무 서버와 초기 운영 환경을 관리한다.

## 6. 보안, 동시성, 패키징과 릴리즈

63. `feat/security-csrf-headers`: CSRF, CSP, security header, upload boundary
64. `feat/security-login-throttling`: login failure rate limit, lockout, audit
65. `feat/security-local-https`: local certificate generation, trust/renewal guidance, Secure cookie
66. `feat/concurrency-idempotency-locks`: duplicate submit, optimistic conflict, lottery/restore lock
67. `feat/access-session-presence`: access code별 active session과 last-seen
68. `test/multi-client-concurrency`: 2~5 browser concurrent reception and WebSocket reconnect
69. `test/large-dataset-performance`: member 1,000, course 50, registration 3,000
70. `test/failure-backup-recovery`: kill/restart, network loss, backup/restore failure
71. `ci/frontend-windows-package`: React, Python, OpenAPI, PyInstaller Windows CI
72. `feat/windows-portable-package`: versioned ZIP, config/runtime folder, first-run guide, checksum
73. `test/windows-portable-smoke`: 일반 사무용 Windows에서 full package 검증 - 수동 확인
74. `docs/operator-guide`: 설치, host launcher, reception, lottery, backup guide
75. `docs/troubleshooting-security`: firewall, certificate, antivirus, restore, log guide
76. `test/prototype-operation-simulation`: 한 회차 전체 운영 rehearsal - 수동 확인

종료 기준: 품질 및 release checklist를 통과하고 `develop`에 Python prototype 기준점을 남긴다. `main` 반영과 release tag는 사용자가 별도로 요청할 때만 진행한다.
