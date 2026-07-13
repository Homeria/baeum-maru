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

## 런타임 설정

개발 환경의 기본 writable 경로는 저장소 루트의 `runtime/`이다. 배포 환경에서는 실행 파일 옆의 `runtime/`을 사용한다. `BAEUM_MARU_RUNTIME_DIR`로 다른 경로를 지정할 수 있다.

```text
runtime/
├─ config/settings.json
├─ config/.env
├─ data/baeum-maru.db
├─ logs/baeum-maru.log
├─ backups/
├─ exports/
├─ imports/
├─ certificates/
└─ tmp/
```

설정 우선순위는 OS 환경변수, `runtime/config/.env`, `runtime/config/settings.json`, 코드 기본값 순이다. 지원 키는 `.env.example`과 `settings.example.json`에서 확인한다.
