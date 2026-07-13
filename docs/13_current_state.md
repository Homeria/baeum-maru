# 현재 구현 상태

## 기준 시점

2026-07-13 기준. Python 백엔드는 읽기 쉬운 layered architecture 보일러플레이트 위에 공통 실행 기반을 순서대로 구현하는 상태다.

## 보존 기준점

- tag: `go-prototype-baseline-2026-07`
- commit: `547977b13d77ffc0dbaa42a4dd4c24829a000d6f`
- 내용: Go/SQLite/Go template/Fyne 기반 기능 검증 구현
- 상태: read-only 참고용이며 활성 코드와 호환 계층을 만들지 않는다.

## 현재 작업 트리

- Go/Fyne/template 코드는 활성 tree에서 제거되어 있다.
- 이전 Python feature-first module, composition container, command/query protocol을 폐기했다.
- 이전 SQLAlchemy model 30개, Alembic migration, schema snapshot과 백엔드 테스트를 폐기했다.
- `backend/app`은 `api/routers`, `schemas`, `services`, `repositories`, `models`, `db`, `core` 수평 계층을 가진다.
- `core/runtime.py`는 한글·공백 경로와 portable 배포를 고려한 writable runtime 디렉터리를 결정한다.
- `core/settings.py`는 기본값, JSON, runtime `.env`, 운영체제 환경변수를 순서대로 병합하고 검증한다.
- `core/logging.py`는 회전 JSON 파일, 콘솔 출력과 민감값 제거를 제공한다.
- architecture test가 하위 계층의 상위 계층 import와 Unit of Work 추상화 재도입을 차단한다.
- `db/base.py`는 Alembic과 model이 공유할 Declarative Base와 제약조건 이름 규칙을 제공한다.
- `db/session.py`는 파일 기반 SQLite engine, 운영 PRAGMA와 자동 commit하지 않는 Session factory를 제공한다.
- `app/models/`의 9개 도메인 파일에 문서 기준 30개 SQLAlchemy table model이 등록되어 있다.
- `20260713_0001_initial_schema` Alembic revision이 빈 DB에 전체 schema와 성별 기준코드를 생성한다.
- 독립 pywebview 제어는 `launcher/`, Excel과 backup 장시간 작업은 `jobs/`로 분리했다.
- Python 3.13과 FastAPI, Pydantic, SQLAlchemy, Alembic, pywebview 의존성 결정은 유지한다.
- 정규화된 데이터 모델 문서는 이후 ORM과 migration을 다시 구현할 설계 기준으로 유지한다.
- `frontend/`의 operator/launcher React/Vite/TypeScript 보일러플레이트는 유지한다.
- GitHub Actions는 비워 두었고 API/frontend 계약 안정화 이후 새로 추가한다.

## 코드 탐색 기준

```text
api/routers/<domain>.py
        ↓
services/<domain>_service.py
        ↓
repositories/<domain>_repository.py
        ↓
models/<domain>.py + db/
```

Schema는 router와 service 사이의 API 입력·응답 계약이다. Router는 request scope Session을 service에 전달하고, repository는 commit하지 않으며 service가 명시적으로 commit 또는 rollback한다.

## 채택한 목표

- Python 3.13, FastAPI, Pydantic v2
- SQLAlchemy 2, Alembic, SQLite WAL
- 명시적인 router-service-repository layered architecture
- React/Vite/TypeScript, TanStack Query, FastAPI WebSocket
- pywebview/WebView2 독립 launcher와 LAN operator server process 분리
- PyInstaller `onedir` portable ZIP
- pytest/HTTPX/Playwright와 Windows package CI

## 바로 다음 작업

`test/sqlite-schema-contract`

실제 SQLite schema snapshot과 대표 위반 SQL을 이용해 FK, UNIQUE, CHECK, partial index, cascade/restrict와 필수 query index를 계약 테스트로 고정한다. `main`은 변경하지 않는다.

## 현재 검증 범위

- runtime 경로, 설정 source 우선순위, 구조화 logging과 민감값 제거 pytest
- 하위 계층의 상위 계층 import를 차단하는 architecture test
- SQLite URL/PRAGMA/FK와 명시적 Session transaction pytest
- 30개 metadata table, optimistic version, Alembic upgrade/check/downgrade와 seed pytest
- Python compile, Ruff와 mypy 통과
- operator/launcher TypeScript typecheck, oxlint, Vitest와 production build 통과
- 실행 가능한 FastAPI API는 아직 존재하지 않음
