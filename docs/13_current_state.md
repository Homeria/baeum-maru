# 현재 구현 상태

## 기준 시점

2026-07-13 기준. Python 백엔드는 구조를 다시 확정하기 위해 실행 구현을 폐기하고, 역할만 설명하는 layered architecture 보일러플레이트 상태다.

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
- 모든 Python 파일에는 실행 코드 없이 해당 파일의 책임을 설명하는 module docstring만 있다.
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

Schema는 router와 service 사이의 API 입력·응답 계약이다. Repository는 commit하지 않고 service가 unit of work를 통해 transaction을 완료한다.

## 채택한 목표

- Python 3.13, FastAPI, Pydantic v2
- SQLAlchemy 2, Alembic, SQLite WAL
- 명시적인 router-service-repository layered architecture
- React/Vite/TypeScript, TanStack Query, FastAPI WebSocket
- pywebview/WebView2 독립 launcher와 LAN operator server process 분리
- PyInstaller `onedir` portable ZIP
- pytest/HTTPX/Playwright와 Windows package CI

## 바로 다음 작업

`feat/config-runtime-foundation`

새 구조에서 설정, writable runtime 경로와 logging을 먼저 구현하고 해당 파일의 테스트를 함께 추가한다. `main`은 변경하지 않는다.

## 현재 검증 범위

- 모든 Python 파일이 module docstring만 포함하는지 검사
- Python compile, Ruff와 mypy 통과
- operator/launcher TypeScript typecheck, oxlint, Vitest와 production build 통과
- 실행 가능한 FastAPI API, DB와 backend pytest는 아직 존재하지 않음
