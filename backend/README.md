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
- `services`: 프레임워크와 저장소에 독립적인 업무 규칙
- `repositories`: parameter binding을 사용하는 명시적인 SQL 조회와 저장
- `db/schema`: 도메인별 `CREATE TABLE`, index와 기준 데이터
- `db/database.py`: 함수형 SQLite connection, transaction과 schema 초기화
- `core`: 설정, runtime 경로, logging, 보안과 공통 예외
- `launcher`: pywebview 런처와 서버 process 제어
- `jobs`: Excel과 backup 같은 장시간 작업

Router는 Pydantic 요청을 검증한 뒤 primitive 또는 표준 라이브러리 dataclass로 풀어 service에 전달한다. Service는 업무 규칙을 처리하며 FastAPI, Pydantic, `sqlite3`에 의존하지 않는다. Repository의 공개 함수가 `get_db_connection()` 또는 `transaction()`을 직접 호출하고 조회 또는 transaction을 끝까지 소유한다.

여러 table과 감사 로그를 원자적으로 바꾸는 작업은 해당 use case를 소유한 Repository 공개 함수 하나가 transaction을 열고 내부 저장 함수를 같은 connection으로 호출한다. Resource event는 transaction commit이 성공한 다음에만 `publish_committed_events()`로 전달한다. 이벤트 전달 실패는 이미 commit된 업무 결과를 실패로 바꾸지 않으며 클라이언트는 재연결 시 REST API를 다시 조회한다.

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

서버 lifespan은 writable runtime 경로를 준비하고 코드 기반 SQLite schema를 초기화한다. DB 객체나 connection을 전역 등록하지 않으며, 실제 connection은 Repository 작업 시 runtime 설정에서 경로를 읽어 열고 닫는다. 모듈 import만으로는 runtime 파일을 만들거나 DB에 연결하지 않는다.

Realtime WebSocket은 동일 출처 요청과 `baeum_maru_session` HttpOnly cookie를 요구한다. 현재 기본 verifier는 모든 연결을 거부하며 실제 access-code session 검증은 인증 도메인 구현에서 연결한다. 연결 성공 시 `ready`, 주기적인 `heartbeat`, commit 이후 `resource_changed`를 전송하고 재연결 또는 event gap에는 REST API 전체 재조회를 요구한다.
