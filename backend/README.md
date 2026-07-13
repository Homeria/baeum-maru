# Backend

Python 3.13, FastAPI와 uv 기반 백엔드 프로젝트다.

```powershell
uv sync --all-groups
uv run uvicorn app.main:app --reload
uv run pytest
uv run ruff format --check .
uv run ruff check .
uv run mypy
```

개발 서버의 상태 확인 API는 `GET /api/v1/health`다.
