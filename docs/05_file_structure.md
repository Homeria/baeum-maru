# 파일 구조

## 기준

백엔드는 프로젝트 소유자가 익숙한 `router → service → repository → database` 수평 계층 구조를 사용한다. 파일을 처음 읽는 사람이 추상화나 조립 코드를 먼저 해석하지 않고도 요청의 흐름을 따라갈 수 있어야 한다.

기존 feature-first `modules/`, `public.py`, composition container와 command/query handler 구조는 폐기했다. Go 구현은 `go-prototype-baseline-2026-07` 태그에만 보존한다.

## 백엔드 구조

```text
backend/
  pyproject.toml
  uv.lock
  app/
    main.py                       FastAPI 생성과 router 등록
    api/
      dependencies.py             인증, pagination 등 HTTP 공통 Depends
      errors.py                   공통 API 오류 응답과 exception handler
      middleware.py               request ID 생성과 응답 header
      realtime.py                 WebSocket 연결, heartbeat와 event broadcast
      routers/                    도메인별 HTTP/WebSocket endpoint
    schemas/                      도메인별 Pydantic 요청·응답
    services/                     프레임워크와 저장소에 독립적인 업무 규칙
    repositories/                 parameterized SQL 조회와 저장
    db/
      database.py                 sqlite3 연결, PRAGMA와 transaction
      schema/                     도메인별 Python DDL과 seed
    core/                         설정, runtime, logging, 예외, 보안
    launcher/                     pywebview와 서버 process 제어
    jobs/                         Excel, backup 등 장시간 작업
  tests/
    conftest.py                   공통 DB와 API fixture
    api/                          endpoint 통합 테스트
    services/                     저장소를 대역으로 둔 업무 규칙 테스트
    repositories/                 query와 DB 제약 테스트
    scenarios/                    핵심 사용자 흐름 테스트
```

공통 runtime/config/logging, sqlite3 연결/transaction, 코드 기반 초기 schema와 FastAPI application 기반이 구현되어 있다. 업무 repository/service와 도메인 router는 이후 세로 슬라이스 브랜치에서 함께 추가한다.

## 기능 탐색 규칙

회원 등록 기능을 찾는 경우 다음 파일을 순서대로 읽는다.

```text
api/routers/members.py
schemas/members.py
services/member_service.py
repositories/member_repository.py
db/schema/members.py
```

강좌, 공간, 신청, 추첨도 같은 이름 규칙을 사용한다. 한 계층의 파일이 커질 때만 같은 이름의 하위 package로 분리하며, 기능마다 임의의 추가 계층을 만들지 않는다.

## 계층 규칙

- router는 요청 검증 결과를 service에 전달하고 응답으로 변환한다.
- Pydantic schema는 router의 API 계약이며 DB 행, DDL 또는 service 입력형으로 사용하지 않는다.
- router는 Pydantic 값을 primitive 또는 표준 라이브러리 dataclass로 풀어 service에 전달한다.
- service는 업무 규칙을 담당하며 FastAPI, Pydantic, `sqlite3`를 import하지 않는다.
- repository 공개 함수는 함수형 DB 유틸리티를 직접 호출해 runtime의 단일 운영 DB connection을 얻고 조회 또는 transaction을 완료한다.
- `db/schema`는 평문 DDL로 table, index와 DB 수준 제약만 표현한다.
- FastAPI `Depends`, `Request`, HTTP status는 router 바깥으로 전달하지 않는다.
- SQL과 `sqlite3.Row`는 repository 바깥으로 노출하지 않는다.
- 여러 table 변경은 use case를 소유한 Repository 공개 함수가 같은 connection과 transaction 안에서 수행한다.
- 감사 로그는 업무 변경과 같은 transaction에 저장하고 resource event는 commit 성공 뒤 전달한다.
- event 전달 실패로 이미 commit된 업무를 rollback하거나 API 실패로 바꾸지 않는다.
- 하위 계층은 상위 계층을 import할 수 없으며 architecture test가 역방향 의존성을 검사한다.
- 별도 저장소 구현이 실제로 필요해지기 전에는 repository protocol이나 generic repository를 만들지 않는다.
- import 시 DB 연결, process 시작과 파일 생성을 수행하지 않는다.

## 프론트엔드와 런타임

`frontend/apps/operator`는 직원 업무 웹 앱, `frontend/apps/launcher`는 pywebview 런처 UI다. writable 데이터는 source 또는 package 내부가 아니라 `runtime/` 아래에 생성한다.
