# 현재 구현 상태

## 기준 시점

2026-07-13 기준. Python 전환 결정과 Go 기준점 보존을 완료했다.

## 보존 기준점

- tag: `go-prototype-baseline-2026-07`
- commit: `547977b13d77ffc0dbaa42a4dd4c24829a000d6f`
- 내용: Go/SQLite/Go template/Fyne 기반 기능 검증 구현
- 상태: read-only 참고용이며 새 기능과 수정은 진행하지 않는다.

## 현재 작업 트리

- 이 문서 branch까지는 Go 코드가 남아 있다.
- 다음 `refactor/python-project-reset`에서 Go/Fyne/template 코드와 Go CI를 제거한다.
- 최신 DB schema, 업무 rule, screen requirement, license policy는 유지한다.
- Python active implementation은 아직 생성되지 않았다.

## 채택한 목표

- Python 3.13, FastAPI, Pydantic v2
- SQLAlchemy 2, Alembic, SQLite WAL
- React/Vite/TypeScript, TanStack Query, SSE
- localhost host console과 LAN operator server 분리
- PyInstaller `onedir` portable ZIP
- pytest/HTTPX/Playwright와 Windows package CI

## 폐기한 계획

- Go `net/http` + Huma v2 전환
- Wails/WebView2 launcher
- Fyne 유지보수와 Go template 점진 전환
- Go와 Python 구현 동시 유지
- 과거 Go DB 파일을 Python schema로 변환하는 migration

## 바로 다음 작업

`refactor/python-project-reset`

이 branch는 Go 구현을 활성 tree에서 제거하고, 최소 FastAPI health endpoint, Python project/lockfile, pytest/Ruff/mypy와 Python CI가 통과하는 새 기준선을 만든다. `main`은 변경하지 않는다.
