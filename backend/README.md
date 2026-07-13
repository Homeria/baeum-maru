# Backend

Python 3.13, FastAPI와 uv를 사용할 배움마루 백엔드 보일러플레이트다.

현재 `app/`의 Python 파일에는 실행 코드가 없으며 각 파일의 책임을 설명하는 module docstring만 있다. 이전 feature-first 구현, SQLAlchemy model, Alembic migration과 자동화 테스트는 새 구조를 확정하기 위해 폐기했다.

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
- `db`: engine, Session과 unit of work
- `core`: 설정, runtime 경로, logging, 보안과 공통 예외
- `launcher`: pywebview 런처와 서버 process 제어
- `jobs`: Excel과 backup 같은 장시간 작업

Repository는 `commit()` 또는 `rollback()`하지 않는다. 여러 저장 작업의 transaction 결과는 service가 unit of work를 통해 결정한다.

## 개발 도구

의존성은 유지하지만 실행 가능한 FastAPI application과 migration은 이후 브랜치에서 다시 구현한다.

```powershell
uv sync --all-groups
uv run ruff format --check .
uv run ruff check .
uv run mypy
```

`uv run uvicorn app.main:app`, Alembic과 pytest 명령은 해당 구현 및 테스트가 추가된 뒤 사용한다.
