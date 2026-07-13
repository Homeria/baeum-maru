# 프로토타입 개발 계획

## 원칙

- 활성 구현을 Python/FastAPI로 전면 교체하며 Go와 Python을 동시에 유지하지 않는다.
- Go 구현은 `go-prototype-baseline-2026-07` 태그에서만 참고한다.
- 실사용 데이터가 없으므로 최신 스키마 하나에서 Alembic 이력을 시작한다.
- 프로젝트 소유자가 읽고 검증할 수 있는 명시적인 코드와 테스트를 우선한다.
- 큰 전환은 항상 buildable하고 검증 가능한 브랜치로 나눈다.
- 모든 작업은 `develop`에만 누적하고 사용자 요청 전에는 `main`을 변경하지 않는다.
- 공통 기반은 수평 계층으로 고정하되, 업무 기능은 DB/application/API/UI/test를 함께 완성하는 세로 슬라이스로 개발한다.
- 서버는 영속 업무 상태를 메모리에 보관하지 않는 stateless modular monolith로 구성한다.

브랜치별 정확한 순서와 수동 검증 지점은 `18_prototype_branch_roadmap.md`를 단일 기준으로 사용한다.

## 단계 0: Python reset

- Go 프로토타입을 annotated tag로 보존한다.
- Go/Fyne/template 코드와 Go 전용 CI를 제거한다.
- Python 3.13, uv, FastAPI, pytest, Ruff와 mypy 의존성 및 읽기 쉬운 backend 보일러플레이트를 만든다.
- pnpm workspace에 operator/launcher React 앱, typecheck, lint, unit test, production build를 구성한다.
- 기존 GitHub Actions를 모두 제거하고 API/frontend 계약이 안정된 뒤 Python/React CI를 새로 추가한다.
- pywebview와 Windows PyInstaller `onedir` 포터블 실행을 조기에 검증한다.

## 단계 1: 실행, architecture와 데이터 기반

- `router → service → repository → database` 수평 계층과 파일 이름 규칙을 먼저 고정한다.
- 각 Python 파일의 책임만 적은 보일러플레이트에서 작은 실행 단위부터 순서대로 구현한다.
- 현재 정규화 스키마를 SQLAlchemy 2 model과 단일 초기 Alembic migration으로 옮긴다.
- FK, unique, check, index, cascade/null 정책을 실제 SQLite 테스트로 고정한다.
- request scope Session, 공통 오류, audit/event 발행 경계를 만든다.
- config, runtime directory, logging, backup filesystem 경계를 분리한다.
- REST/WebSocket, React static serving, pywebview와 Windows `onedir` 실행 가능성을 업무 기능보다 먼저 검증한다.

## 단계 2: 접근과 기관 환경

- access code, session, role과 만료/폐기 정책을 먼저 구현한다.
- 직원 로그인 화면과 런처 접속 관리를 같은 session 계약에 연결한다.
- 기관 정보와 building/floor/space/role 환경 구성을 실제 화면까지 완성한다.

## 단계 3: 강좌, 회원과 접수

- 강좌 기준정보와 개설 강좌, 복수 시간표를 application/API/UI까지 완성한다.
- 회원 등록/수정/검색과 신청 이력 조회를 구현한다.
- 회원과 여러 신청을 하나의 transaction으로 저장하는 reception submission을 별도 command로 만든다.
- commit 이후 WebSocket event로 다른 직원의 query cache를 갱신한다.

## 단계 4: 추첨, 출석과 운영

- 추첨, 대기자, 출석, Excel 입출력, backup/restore를 각 세로 슬라이스로 구현한다.
- audit와 장기 작업 상태를 직원 화면과 런처에서 조회한다.
- 핵심 업무 전체를 실제 SQLite integration test로 연결한다.

## 단계 5: 런처 운영 고도화

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
