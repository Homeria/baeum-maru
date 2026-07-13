# 현재 구현 상태

## 기준 시점

2026-07-13 기준. Go 활성 구현 제거와 Python/React 보일러플레이트 구성을 완료했다.

## 보존 기준점

- tag: `go-prototype-baseline-2026-07`
- commit: `547977b13d77ffc0dbaa42a4dd4c24829a000d6f`
- 내용: Go/SQLite/Go template/Fyne 기반 기능 검증 구현
- 상태: read-only 참고용이며 새 기능과 수정은 진행하지 않는다.

## 현재 작업 트리

- Go/Fyne/template 코드, Go module, 기존 package script와 GitHub Actions를 제거했다.
- `backend/`는 Python 3.13, uv, FastAPI health API, pytest, Ruff, mypy 기준선을 가진다.
- `frontend/`는 pnpm workspace와 operator/launcher React/Vite/TypeScript 앱을 가진다.
- 두 frontend 앱은 Vitest/Testing Library, oxlint, TypeScript build가 구성되어 있다.
- GitHub Actions는 비워 두었고 Python API/frontend 계약 안정화 이후 새로 추가한다.
- 최신 DB schema, 업무 rule, screen requirement, license policy는 유지한다.

## 채택한 목표

- Python 3.13, FastAPI, Pydantic v2
- SQLAlchemy 2, Alembic, SQLite WAL
- React/Vite/TypeScript, TanStack Query, FastAPI WebSocket
- pywebview/WebView2 독립 launcher와 LAN operator server process 분리
- PyInstaller `onedir` portable ZIP
- pytest/HTTPX/Playwright와 Windows package CI

## 폐기한 계획

- Go `net/http` + Huma v2 전환
- localhost 기본 브라우저 host console
- Wails launcher
- Fyne 유지보수와 Go template 점진 전환
- Go와 Python 구현 동시 유지
- 과거 Go DB 파일을 Python schema로 변환하는 migration

## 바로 다음 작업

`feat/python-config-runtime`

이 branch는 portable runtime path, Pydantic settings, 환경별 설정, structured logging과 writable directory 경계를 구성한다. `main`은 변경하지 않는다.

## 현재 로컬 검증

- `uv lock --check`, Ruff, mypy, pytest 통과
- operator/launcher TypeScript typecheck와 oxlint 통과
- operator/launcher Vitest와 Vite production build 통과
