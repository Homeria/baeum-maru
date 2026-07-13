# 현재 구현 상태

## 기준 시점

2026-07-13 기준. Python/React 보일러플레이트, config/runtime와 application architecture 기반을 완료했다.

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
- backend는 개발/배포 포터블 runtime 경로와 writable directory를 자동 구성한다.
- 설정은 JSON, runtime `.env`, OS 환경변수를 Pydantic으로 병합·검증한다.
- application logger는 JSON rotating file과 console을 제공하고 주요 secret/개인정보 key를 마스킹한다.
- FastAPI lifespan은 불변 `RuntimeContext`를 조립하며 업무 상태를 프로세스 전역에 저장하지 않는다.
- `app/composition.py`가 concrete handler와 router를 불변 `ApplicationContainer`로 조립한다.
- health 기능은 첫 feature slice인 `modules/system` 안에서 API/schema/application/public 경계로 동작한다.
- command/query protocol과 AST architecture test가 layer 역참조와 feature 내부 직접 import를 차단한다.
- Python 초기 schema의 30개 table, column, CHECK/UNIQUE/index와 FK 삭제 정책을 확정했다.
- location은 building floor FK, course schedule은 공간 FK를 가지며 접속 코드 만료 상태는 시각에서 계산한다.
- 기능별 SQLAlchemy model 30개, 공통 metadata registry와 SQLite session factory를 구현했다.
- `20260713_0001` initial Alembic revision은 빈 DB upgrade/downgrade, metadata diff와 성별 seed 검사를 통과한다.
- mutable model의 `version`은 SQLAlchemy optimistic concurrency mapper에 연결되어 있다.
- 실제 SQLite의 컬럼, PK, FK 삭제 정책, UNIQUE, CHECK와 index는 승인된 schema contract snapshot으로 고정되어 있다.
- 대표 무결성 위반과 partial UNIQUE, CASCADE/RESTRICT 동작을 실제 SQL로 검증한다.
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

`feat/sqlalchemy-unit-of-work`

이 branch는 HTTP request와 launcher/background task가 같은 transaction 원칙을 사용하도록 session과 unit of work 경계를 정의한다. commit/rollback 소유권과 repository가 session 생명주기를 소유하지 않는 규칙을 테스트로 고정한다. `main`은 변경하지 않는다.

## 현재 로컬 검증

- `uv lock --check`, Ruff, mypy, pytest 30개 통과
- operator/launcher TypeScript typecheck와 oxlint 통과
- operator/launcher Vitest와 Vite production build 통과
- 실제 Uvicorn process에서 `/api/v1/health`, runtime directory와 JSON file log 생성을 확인
