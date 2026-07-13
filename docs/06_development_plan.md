# 프로토타입 개발 계획

## 원칙

- 활성 구현을 Python/FastAPI로 전면 교체하며 Go와 Python을 동시에 유지하지 않는다.
- Go 구현은 `go-prototype-baseline-2026-07` 태그에서만 참고한다.
- 실사용 데이터가 없으므로 최신 스키마 하나에서 Alembic 이력을 시작한다.
- 프로젝트 소유자가 읽고 검증할 수 있는 명시적인 코드와 테스트를 우선한다.
- 큰 전환은 항상 buildable하고 검증 가능한 브랜치로 나눈다.
- 모든 작업은 `develop`에만 누적하고 사용자 요청 전에는 `main`을 변경하지 않는다.

브랜치별 정확한 순서와 수동 검증 지점은 `18_prototype_branch_roadmap.md`를 단일 기준으로 사용한다.

## 단계 0: Python reset

- Go 프로토타입을 annotated tag로 보존한다.
- Go/Fyne/template 코드와 Go 전용 CI를 제거한다.
- Python 3.13, uv, FastAPI health endpoint, pytest, Ruff, mypy CI로 빈 기반을 만든다.
- React workspace와 Windows PyInstaller `onedir` 포터블 실행을 조기에 검증한다.

## 단계 1: 데이터와 application 기반

- 현재 정규화 스키마를 SQLAlchemy 2 model과 단일 초기 Alembic migration으로 옮긴다.
- FK, unique, check, index, cascade/null 정책을 실제 SQLite 테스트로 고정한다.
- request scope session, unit of work, 공통 오류, audit/event 발행 경계를 만든다.
- config, runtime directory, logging, backup filesystem 경계를 분리한다.

## 단계 2: 업무 모듈

- identity, members, locations, courses, registrations, lottery, attendance, operations 순서로 구현한다.
- 회원과 여러 신청을 하나의 transaction으로 저장하는 reception submission을 별도 use case로 만든다.
- 기존 Go 코드를 줄 단위로 번역하지 않고 스키마 문서와 업무 규칙에서 Python 구현을 작성한다.
- 각 모듈은 repository integration test와 application test를 가진다.

## 단계 3: REST API와 실시간 갱신

- `/api/v1`, 공통 오류, pagination/filter, OpenAPI 규약을 먼저 확정한다.
- 접속 코드 session, 역할 권한, CSRF, throttling을 server side에서 강제한다.
- 각 업무 모듈의 REST endpoint와 commit 이후 WebSocket domain event를 추가한다.
- OpenAPI에서 TypeScript client를 생성하고 계약 차이를 CI에서 검사한다.

## 단계 4: React 업무 화면

- 디자인 token과 업무용 primitive를 만든 뒤 로그인과 app shell을 구성한다.
- 접수, 회원, 강좌, 신청, 추첨, 출석, Excel/백업 순서로 화면을 구현한다.
- TanStack Query와 WebSocket을 연결하고 폼 편집 중 변경 충돌 UX를 제공한다.
- 한글 입력, 키보드, 좁은 화면, 다중 브라우저를 자동/수동 검증한다.

## 단계 5: pywebview 호스트 런처

- pywebview/WebView2 독립 창, React launcher app과 검증된 Python bridge를 만든다.
- 업무 서버 시작/중지/재시작, bind address, 실제 접속 URL, 접속 코드, 로그, 백업, 초기 공간 설정을 제공한다.
- launcher bridge가 원격 자산이나 LAN에서 호출되지 않는지 자동 테스트한다.
- 실행 파일을 켰을 때 독립 런처 창만 열고 업무 서버는 정지 상태를 유지한다.

## 단계 6: 보안, 패키징, 운영 검증

- HTTPS, secure cookie, CSRF, CSP, 로그인 실패 제한을 검증한다.
- 동일 회원 동시 접수, 추첨 잠금, 중복 제출, 재기동과 WebSocket 재연결을 테스트한다.
- WebView2 Runtime 누락, 한글 IME와 pywebview 창 lifecycle을 실제 Windows에서 검증한다.
- PyInstaller `onedir` 포터블 ZIP을 Windows CI와 일반 사무용 노트북에서 확인한다.
- 회원 1,000명, 강좌 50개, 신청 3,000건과 2~5개 브라우저로 운영 리허설을 수행한다.
- 설치, 운영, 백업, 장애 대응 가이드를 작성한다.
