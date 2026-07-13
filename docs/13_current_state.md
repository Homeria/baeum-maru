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
- 실제 SQLite table/index SQL fingerprint, 인덱스 38개와 FK 35개 정책을 schema contract test로 고정했다.
- 대표 CHECK/UNIQUE 위반과 aggregate CASCADE/업무 이력 RESTRICT를 실제 SQL로 검증한다.
- `create_app()`과 lifespan이 runtime/config/logging, Alembic upgrade, SQLite engine과 Session factory를 순서대로 준비한다.
- `/api/v1` 아래에 health endpoint와 OpenAPI를 제공하고 request ID, 공통 오류 응답과 pagination 계약을 공유한다.
- `Depends(get_db)`는 요청마다 Session을 열고 닫되 자동 commit하지 않아 service transaction 소유권을 유지한다.
- `record_audit()`은 민감 metadata를 거부하고 업무 변경과 같은 Session에 append-only 감사 row를 추가한다.
- `ResourceEvent`와 `publish_committed_events()`는 commit 이후 개인정보 없는 변경 신호를 best-effort로 전달한다.
- 독립 pywebview 제어는 `launcher/`, Excel과 backup 장시간 작업은 `jobs/`로 분리했다.
- Python 3.13과 FastAPI, Pydantic, SQLAlchemy, Alembic, pywebview 의존성 결정은 유지한다.
- 정규화된 데이터 모델 문서는 ORM, migration과 schema contract의 source of truth로 유지한다.
- `frontend/`의 operator/launcher React/Vite/TypeScript 보일러플레이트는 유지한다.
- GitHub Actions는 `develop` PR/push에서 Python과 React 기본 품질 검사를 각각 실행한다.

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

`feat/realtime-websocket-foundation`

인증 가능한 WebSocket 연결, heartbeat, resource event broadcast와 재연결 계약을 구현한다. 아직 인증 도메인이 없으므로 연결 인증의 인터페이스와 테스트 대역을 먼저 두고 실제 access code session은 후속 브랜치에서 연결한다. `main`은 변경하지 않는다.

## 현재 검증 범위

- runtime 경로, 설정 source 우선순위, 구조화 logging과 민감값 제거 pytest
- 하위 계층의 상위 계층 import를 차단하는 architecture test
- SQLite URL/PRAGMA/FK와 명시적 Session transaction pytest
- 30개 metadata table, optimistic version, Alembic upgrade/check/downgrade와 seed pytest
- SQLite schema fingerprint, 필수 index/FK 정책, 대표 제약 위반과 삭제 정책 pytest
- Python compile, Ruff와 mypy 통과
- operator/launcher TypeScript typecheck, oxlint, Vitest와 production build 통과
- FastAPI lifespan, health, OpenAPI, request ID, 공통 오류와 pagination API 계약 pytest
- `develop` PR/push에서 backend format/lint/typecheck/test와 frontend typecheck/lint/test/build CI
- 업무 변경과 감사 로그의 원자적 commit/rollback, commit 이후 event 전달과 실패 격리 pytest
- 업무 도메인 endpoint, 인증과 WebSocket은 아직 구현하지 않음
