# Backend

Python 3.13, FastAPI와 uv를 사용할 배움마루 백엔드 보일러플레이트다.

현재 공통 runtime/config/logging, 표준 `sqlite3` 연결과 transaction, 도메인별 Python DDL, FastAPI 공통 실행 기반이 구현되어 있다. 업무 도메인 API는 이후 브랜치에서 `router → service → repository` 흐름으로 추가한다.

## 읽는 순서

하나의 기능은 아래 순서로 읽는다.

```text
app/api/routers/<domain>.py
        ↓
app/services/<domain>_service.py
        ↓
app/repositories/<domain>_repository.py
        ↓
app/db/schema/<domain>.py + app/db/database.py
```

- `routers`: HTTP/WebSocket 입출력과 service 호출
- `schemas`: Pydantic 요청·응답 형식
- `services`: 업무 규칙과 transaction 흐름
- `repositories`: parameter binding을 사용하는 명시적인 SQL 조회와 저장
- `db/schema`: 도메인별 `CREATE TABLE`, index와 기준 데이터
- `db/database.py`: 연결 PRAGMA와 request scope connection, transaction
- `core`: 설정, runtime 경로, logging, 보안과 공통 예외
- `launcher`: pywebview 런처와 서버 process 제어
- `jobs`: Excel과 backup 같은 장시간 작업

Router는 `Depends(get_db)`로 `sqlite3.Connection`을 받아 service에 전달한다. Repository는 전달받은 연결에서 parameterized SQL만 실행하며 `commit()`, `rollback()`, `close()`하지 않는다. Service가 하나의 업무 흐름을 transaction으로 묶어 성공 시 commit하고 실패 시 rollback한다.

감사 로그는 업무 변경과 같은 연결과 transaction에 추가해 함께 commit한다. Resource event는 transaction commit이 성공한 다음에만 `publish_committed_events()`로 전달한다. 이벤트 전달 실패는 이미 commit된 업무 결과를 실패로 바꾸지 않으며 클라이언트는 재연결 시 REST API를 다시 조회한다.

## 개발 도구

의존성을 설치하고 전체 백엔드 검사를 실행하려면 다음 명령을 사용한다.

```powershell
uv sync --all-groups
uv run ruff format --check .
uv run ruff check .
uv run mypy
uv run pytest
```

개발 서버는 다음 명령으로 실행한다.

```powershell
uv run uvicorn app.main:app --reload
```

- health: `GET /api/v1/health`
- OpenAPI: `/api/v1/openapi.json`
- Swagger UI: `/api/docs`
- realtime: `WS /api/v1/events/ws`

서버 lifespan은 writable runtime 경로를 준비한 뒤 코드 기반 SQLite schema를 초기화하고 request scope 연결을 제공한다. 모듈 import만으로는 runtime 파일을 만들거나 DB에 연결하지 않는다.

Realtime WebSocket은 동일 출처 요청과 `baeum_maru_session` HttpOnly cookie를 요구한다. 현재 기본 verifier는 모든 연결을 거부하며 실제 access-code session 검증은 인증 도메인 구현에서 연결한다. 연결 성공 시 `ready`, 주기적인 `heartbeat`, commit 이후 `resource_changed`를 전송하고 재연결 또는 event gap에는 REST API 전체 재조회를 요구한다.
