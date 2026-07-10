# 프로토타입 브랜치 로드맵

## 목적

기능 MVP를 `Go + net/http + Huma v2 + SQLite + React/Vite/TypeScript + Wails v2` 기반 프로토타입으로 전환하기 위한 전체 브랜치 순서다. 다음 브랜치를 추천하거나 구현하기 전에 이 문서를 먼저 확인한다.

## 운영 규칙

- 모든 작업 브랜치는 최신 `develop`에서 만들고 PR과 CI 통과 후 `develop`에 merge한다.
- 순서를 임의로 건너뛰지 않는다. spike가 실패하면 이후 계획을 진행하지 않고 기술 결정을 다시 기록한다.
- 모든 프로토타입 단계는 `develop`에만 모은다. 사용자 명시 요청 전에는 `main`을 변경하거나 PR을 열지 않는다.
- 수동 확인 단계는 사용자의 확인과 피드백을 받아야 완료된다.
- 실제 구현 중 더 적절한 분리가 발견되면 문서 PR에서 근거와 함께 순서를 조정한다.

## 현재 위치

- 기능 MVP 기준선: `develop`의 `c3460aa`
- 최근 완료: `test/prototype-baseline-characterization`
- 다음 예정: `refactor/launcher-core`
- 병합 정책: 각 작업은 CI 통과 후 `develop`에만 병합한다.

## 1. 기준선과 아키텍처

1. `docs/prototype-branch-roadmap`: 전체 로드맵과 진행 규칙 확정
2. `test/prototype-baseline-characterization`: 회원, 신청, 추첨, 출석, Excel, 복구의 기존 행동 고정
3. `refactor/launcher-core`: Fyne에서 서버 lifecycle, 네트워크, 로그 제어 분리
4. `spike/wails-launcher-prototype`: WebView2, 한글 입력, 탭 유지, 빌드 크기 검증 - 수동 확인
5. `refactor/platform-boundaries`: config, DB, migration, logging, filesystem 경계 정리
6. `refactor/unit-of-work`: 여러 저장소를 묶는 application transaction 도입
7. `refactor/application-events`: 감사 로그와 SSE 발행을 HTTP handler 밖으로 이동

종료 기준: 기존 기능 테스트가 유지되고 Wails 사용 가능성이 실제 Windows에서 확인된다.

## 2. 기능 중심 모듈형 모놀리스

8. `refactor/identity-module`: 사용자, 접속 코드, 세션, 권한 모듈화
9. `refactor/member-module`: 회원 command/query/port/SQLite adapter 분리
10. `refactor/location-floor-reference`: 층 FK 정책 확정과 기준 스키마 수정
11. `refactor/location-module`: 건물, 층, 공간, 역할 모듈화
12. `refactor/course-module`: 회차, 분야, 과목, 강사, 개설, 시간표 모듈화
13. `refactor/registration-module`: 접수, 제한 규칙, 상태 전이 모듈화
14. `refactor/reception-submission-usecase`: 회원과 복수 신청을 원자적으로 저장하는 유스케이스
15. `refactor/lottery-module`: 추첨 실행, 결과, 대기자 승격 모듈화
16. `refactor/attendance-module`: 출석 회차와 기록 모듈화
17. `refactor/operation-module`: Excel, 백업, 복구, 작업 이력 모듈화
18. `refactor/app-composition-root`: 런타임 조립을 모듈 단위로 단순화
19. `refactor/remove-global-layer-packages`: 기존 전역 service/repository/domain 잔여 제거

종료 기준: 기능별 모듈이 transport와 저장 기술에 독립적이고 기존 업무 흐름이 동일하게 동작한다.

## 3. Huma REST API

20. `feat/huma-api-foundation`: `/api/v1`, RFC 오류, OpenAPI 문서, health
21. `feat/api-auth-sessions`: 로그인, 로그아웃, 현재 사용자, 권한
22. `feat/api-members`: 회원 검색, 상세, 등록, 수정
23. `feat/api-locations`: 건물, 층, 공간, 역할 API
24. `feat/api-course-reference-data`: 회차, 분야, 과목, 강사, 시간대 API
25. `feat/api-course-offerings`: 강좌 개설과 복수 시간표 API
26. `feat/api-reception-submissions`: 회원과 복수 강좌 접수 API
27. `feat/api-registrations`: 신청 조회, 취소, 확정, 상태 이력
28. `feat/api-lottery-runs`: 추첨 실행과 결과 조회
29. `feat/api-attendance`: 출석 회차와 기록
30. `feat/api-excel-operations`: Excel 가져오기와 내보내기
31. `feat/api-backup-restore`: 백업 목록, 생성, 복구 예약
32. `feat/api-settings-audit`: 설정과 감사 로그
33. `feat/api-realtime-events`: `/api/v1/events`와 domain scope
34. `test/openapi-contract`: endpoint, DTO, 오류, 하위 호환성 검사
35. `ci/api-contract`: OpenAPI 차이와 Go API 테스트를 PR에서 검사

종료 기준: React가 기존 HTML 경로 없이 모든 업무 기능을 호출할 수 있고 OpenAPI가 실제 응답과 일치한다.

## 4. React 웹

36. `feat/react-web-foundation`: Vite/TypeScript, embed, 개발/production 빌드
37. `feat/web-generated-api-client`: OpenAPI 기반 TypeScript client
38. `feat/web-design-system`: 토큰, 버튼, 입력, 표, 모달, 알림, 아이콘
39. `feat/web-auth-shell`: 로그인, 라우팅, 권한 메뉴, app shell
40. `feat/web-realtime-query-sync`: TanStack Query와 SSE 연결
41. `feat/web-reception-member-flow`: 회원 검색, 선택, 신규 입력
42. `feat/web-reception-course-picker`: 검색 가능한 복수 강좌 선택
43. `feat/web-reception-submit`: 검증, 원자적 저장, 충돌 UX - 수동 확인
44. `feat/web-member-management`: 회원 목록, 상세, 수정, 신청 이력
45. `feat/web-location-management`: 건물, 층, 공간, 역할 관리
46. `feat/web-course-operations`: 기준 데이터와 강좌 개설, 복수 시간표 - 수동 확인
47. `feat/web-registration-dashboard`: 신청 검색, 필터, 상태 관리
48. `feat/web-lottery`: 추첨 준비, 실행, 결과
49. `feat/web-attendance`: 출석 입력과 조회
50. `feat/web-operations`: Excel, 백업, 설정, 감사 로그
51. `test/web-e2e-accessibility`: Playwright, 키보드, 반응형, 한글 입력
52. `refactor/remove-template-web`: React 대체 완료 후 Go template 제거

종료 기준: 직원이 브라우저에서 한 회차의 핵심 업무를 수행하고 다른 사용자 변경이 안전하게 반영된다.

## 5. Wails 런처

53. `feat/wails-launcher-foundation`: 실제 Wails 진입점과 공통 React UI
54. `feat/wails-dashboard-server-control`: 서버 상태, 시작/중지, 접속 주소
55. `feat/wails-access-management`: 코드 발급, 연장, 폐기, 로그인 현황
56. `feat/wails-location-setup`: 초기 건물, 층, 공간, 역할 설정
57. `feat/wails-logs-backup-settings`: 로그, 백업, 네트워크, 기관 설정 - 수동 확인
58. `feat/wails-webview2-fallback`: 런타임 감지, 오프라인 설치, 콘솔 fallback
59. `refactor/remove-fyne-launcher`: Wails 검증 후 Fyne와 CGO 의존성 제거

종료 기준: 호스트 담당자가 Wails 런처만으로 서버와 운영 환경을 관리하고 장애 시 fallback을 사용할 수 있다.

## 6. 실사용 안정화와 릴리즈

60. `feat/security-csrf-headers`: CSRF, CSP 등 보안 헤더
61. `feat/security-login-throttling`: 로그인 실패 제한과 감사
62. `feat/security-local-https`: 로컬 인증서 생성, 신뢰, 갱신
63. `feat/concurrency-idempotency-locks`: 중복 제출, 동시 수정, 추첨 잠금
64. `feat/access-session-presence`: 접속 코드별 활성 세션 현황
65. `test/multi-client-concurrency`: 2~5개 브라우저 동시 접수
66. `test/large-dataset-performance`: 회원 1,000명, 강좌 50개, 신청 3,000건
67. `test/failure-backup-recovery`: 종료, 네트워크 단절, 백업/복구 실패
68. `ci/frontend-wails-windows`: React와 Wails Windows CI
69. `feat/windows-portable-package`: WebView2 수단과 fallback 포함 ZIP
70. `test/windows-portable-smoke`: 일반 사무용 Windows 실기기 검증 - 수동 확인
71. `docs/operator-guide`: 설치, 접수, 추첨, 백업 사용자 가이드
72. `docs/troubleshooting-security`: 방화벽, 인증서, WebView2, 복구 가이드
73. `test/prototype-operation-simulation`: 한 회차 전체 운영 리허설 - 수동 확인

종료 기준: 품질 및 릴리즈 체크리스트를 모두 통과하고 `develop`에 프로토타입 기준점을 남긴다. `main` 반영과 태그는 사용자가 별도로 요청할 때만 진행한다.
