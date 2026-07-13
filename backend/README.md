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

`app/composition.py`가 유일한 composition root이며 `app/main.py`는 이 경계만 import한다. 업무 코드는 `app/modules/<feature>/`에 기능별로 모으고, `pytest`에 포함된 architecture test가 domain/application의 framework·persistence 역참조와 비공개 모듈 간 import를 차단한다.

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

## 데이터베이스 migration

새 DB와 schema 변경은 `Base.metadata.create_all()`이 아니라 Alembic으로만 적용한다. 별도 설정이 없으면 저장소의 `runtime/data/baeum-maru.db`를 사용한다.

```powershell
uv run alembic -c alembic.ini upgrade head
uv run alembic -c alembic.ini check
```

현재 기준선은 `20260713_0001_initial_schema.py` 한 개이며 업무 테이블 30개와 성별 코드 3개를 생성한다.
