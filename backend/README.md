# Backend

Python 3.13, FastAPI와 uv를 사용할 배움마루 백엔드 보일러플레이트다.

현재 공통 runtime/config/logging, SQLAlchemy Base/engine/Session factory, 30개 업무 model과 단일 Alembic initial revision, FastAPI 공통 실행 기반이 구현되어 있다. 업무 도메인 API는 이후 브랜치에서 `router → service → repository` 흐름으로 추가한다.

## 읽는 순서

하나의 기능은 아래 순서로 읽는다.

```text
app/api/routers/<domain>.py
        ↓
app/services/<domain>_service.py
        ↓
app/repositories/<domain>_repository.py
        ↓
app/models/<domain>.py + app/db/
```

- `routers`: HTTP/WebSocket 입출력과 service 호출
- `schemas`: Pydantic 요청·응답 형식
- `services`: 업무 규칙과 transaction 흐름
- `repositories`: SQLAlchemy 조회와 저장
- `models`: SQLAlchemy table model
- `db`: engine과 request scope Session
- `core`: 설정, runtime 경로, logging, 보안과 공통 예외
- `launcher`: pywebview 런처와 서버 process 제어
- `jobs`: Excel과 backup 같은 장시간 작업

Router는 `Depends(get_db)`로 Session을 받아 service에 전달한다. Repository는 전달받은 Session으로 query, `add()`와 `flush()`만 수행하며 `commit()`, `rollback()`, `close()`하지 않는다. Service가 업무 흐름의 성공 시 `commit()`, 실패 시 `rollback()`을 명시적으로 수행한다.

## 개발 도구

의존성을 설치하고 전체 백엔드 검사를 실행하려면 다음 명령을 사용한다.

```powershell
uv sync --all-groups
uv run ruff format --check .
uv run ruff check .
uv run mypy
uv run pytest
uv run alembic upgrade head
```

개발 서버는 다음 명령으로 실행한다.

```powershell
uv run uvicorn app.main:app --reload
```

- health: `GET /api/v1/health`
- OpenAPI: `/api/v1/openapi.json`
- Swagger UI: `/api/docs`

서버 lifespan은 writable runtime 경로를 준비하고 Alembic revision을 `head`까지 적용한 뒤 SQLite engine과 request scope Session factory를 생성한다. 모듈 import만으로는 runtime 파일을 만들거나 DB에 연결하지 않는다.
