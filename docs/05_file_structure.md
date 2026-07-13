# 파일 구조

## 전환 전 구조

Go 기준점은 `go-prototype-baseline-2026-07` 태그에만 보존한다. `cmd/`, `internal/`, Go template 자산, `go.mod`, `go.sum`과 Go/Fyne workflow는 활성 트리에서 제거했다.

## 목표 구조

```text
backend/
  pyproject.toml              uv project와 Python 품질 도구 설정
  uv.lock                    Python 의존성 잠금
  .python-version            Python 3.13 개발 기준
  app/
    main.py                   FastAPI operator app 조립
    launcher_main.py          pywebview launcher entry point
    core/                     runtime path, config, security, errors, logging, events
    db/                       SQLAlchemy base, session, unit of work
    launcher/                 bridge, server lifecycle, network, diagnostics
    modules/
      identity/
      members/
      locations/
      courses/
      registrations/
      lottery/
      attendance/
      operations/
  alembic/
    versions/                 최신 기준 스키마부터 시작하는 migration
  tests/
    unit/ integration/ contract/

frontend/
  package.json               workspace 공통 command
  pnpm-workspace.yaml
  pnpm-lock.yaml             frontend 의존성 잠금
  apps/
    operator/                접수 직원과 업무 관리자 React 앱
    launcher/                pywebview 독립 런처 React 앱
  packages/
    ui/                       검증된 공통 primitive와 디자인 token
    api-client/               OpenAPI 생성 타입과 client

scripts/
  package_windows.ps1
  smoke_windows.ps1

docs/
.github/workflows/
```

GitHub Actions는 Python/React 계약이 안정되기 전까지 비워 두고 로드맵의 CI branch에서 새로 구성한다.

## 기능 우선 모듈

```text
modules/members/
  api.py                      FastAPI router와 HTTP DTO 변환
  schemas.py                  Pydantic request/response model
  service.py                  application use case
  domain.py                   업무 entity/value/rule
  ports.py                    repository protocol
  repository.py               SQLAlchemy adapter
  models.py                   SQLAlchemy persistence model
```

모듈이 작을 때 파일을 억지로 모두 만들지 않는다. 하지만 dependency 방향과 transaction 경계는 유지한다.

루트에 전역 `routers/`, `services/`, `repositories/`를 만들지 않는다. 회원 코드는 `modules/members/`, 강좌 코드는 `modules/courses/`처럼 기능별로 모으고, 모듈이 실제로 커질 때만 `api/`, `application/`, `domain/`, `infrastructure/` 하위 디렉터리로 확장한다.

## 런타임 파일

```text
runtime/
  config/settings.json
  config/.env
  data/baeum-maru.db
  logs/baeum-maru.log
  backups/
  exports/
  imports/
  certificates/
  tmp/
```

이 디렉터리는 source나 PyInstaller bundle에 포함하지 않는다. 개발 시 저장소 루트, 배포 시 실행 파일 옆에 만들며 `BAEUM_MARU_RUNTIME_DIR`로 재정의할 수 있다.

## 경계 규칙

- FastAPI router는 HTTP 처리와 application service 호출만 담당한다.
- Pydantic schema와 SQLAlchemy model을 분리한다.
- application service는 FastAPI `Request`, `Depends`, React 타입을 import하지 않는다.
- domain rule은 DB session이나 HTTP status code를 알지 않는다.
- repository protocol과 unit of work를 통해 persistence를 교체할 수 있게 한다.
- SQLAlchemy query가 router와 React layer로 새지 않게 한다.
- launcher bridge는 업무 모듈의 public application API만 사용하고 모든 입력을 Python에서 다시 검증한다.
- React는 생성된 API client를 통해서만 backend와 통신한다.
- 공통 `packages/ui`에는 두 앱에서 실제로 반복된 안정된 컴포넌트만 옮긴다.

## 의존 방향

```text
FastAPI / React adapter
        ↓
application service
        ↓
domain + port
        ↑
SQLAlchemy / filesystem / Excel adapter
```

Python의 import 편의 때문에 계층을 우회하지 않는다. 순환 import가 생기면 전역 조립으로 덮지 않고 모듈 책임과 protocol 위치를 다시 검토한다.
