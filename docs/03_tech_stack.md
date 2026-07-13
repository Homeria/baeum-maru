# 배움마루 기술 스택

이 문서는 Python 전환 이후 프로토타입이 따라야 할 채택 기술 스택이다. 버전은 `pyproject.toml`, `uv.lock`, `package.json`과 lockfile에서 고정한다.

## 결정

| 영역 | 선택 | 역할 |
|---|---|---|
| 언어 | Python 3.13.x | HTTP API, 업무 규칙, 파일/서버 제어 |
| 백엔드 | FastAPI, Uvicorn | REST API, OpenAPI, 정적 자산, ASGI 실행 |
| 검증/설정 | Pydantic v2, pydantic-settings | 요청/응답 DTO와 환경/JSON 설정 검증 |
| DB | SQLite WAL, SQLAlchemy 2 | 호스트 로컬 데이터와 명시적 repository 구현 |
| migration | Alembic | 최신 기준 스키마와 이후 변경 이력 |
| 웹 UI | React, TypeScript, Vite | 직원 업무 화면과 호스트 관리 콘솔 |
| 서버 상태 | TanStack Query, SSE | 조회 cache와 서버 변경 이벤트 기반 갱신 |
| 폼 | React Hook Form, Zod | 복잡한 입력 상태와 즉시 사용자 피드백 |
| 스타일 | CSS variables, CSS Modules, Lucide icons | 조용하고 밀도 높은 기관 업무 UI |
| Excel | openpyxl | Excel 가져오기와 내보내기 |
| Python 도구 | uv, `pyproject.toml`, `uv.lock` | 재현 가능한 개발 환경과 의존성 잠금 |
| Node 도구 | pnpm workspace, lockfile | operator/host/shared React package 관리 |
| 백엔드 검증 | pytest, pytest-asyncio, HTTPX | 도메인, DB, API, 비동기 흐름 검증 |
| 프론트 검증 | Vitest, Testing Library, Playwright | 컴포넌트와 다중 브라우저 업무 흐름 검증 |
| 정적 검사 | Ruff, mypy | formatting, lint, 타입 검사 |
| Windows 배포 | PyInstaller `onedir`, portable ZIP | Python 미설치 PC에서 실행 |
| CI/CD | GitHub Actions | Python/API/UI/Windows 패키지 검사 |

## 선택 원칙

- 목표 환경은 Windows 사무용 노트북 한 대와 내부망 사용자 2~5명이다.
- 프로젝트 소유자가 코드를 읽고 테스트와 업무 검증에 직접 참여할 수 있어야 한다.
- 직원 PC에는 앱을 설치하지 않고 브라우저를 사용한다.
- 운영 PC에는 Python, Node.js, uv, pnpm, Docker Desktop, C 컴파일러를 요구하지 않는다.
- 개발 편의보다 최종 포터블 패키지의 재현성과 Windows 호환성을 우선한다.
- Python 성능과 패키지 크기 증가는 목표 동시 사용자와 유지보수 이점을 고려해 수용한다.
- `onefile`보다 시작과 진단이 예측 가능한 `onedir`를 기본 배포 단위로 사용한다.

## 런타임 구성

```text
배움마루.exe
└─ Python 프로세스
   ├─ 호스트 제어면: 127.0.0.1 전용
   │  └─ 서버 상태, 네트워크, 접속 코드, 로그, 백업, 설정
   └─ 업무 서버: 설정된 host:port
      ├─ /api/v1 REST API와 /api/v1/events SSE
      └─ React 직원 웹 자산

SQLite / data / backups / exports / logs는 실행 파일 외부에 저장
```

호스트 제어면과 업무 서버는 같은 Python 프로세스 안에서 명시적으로 분리한다. 제어 API는 LAN 인터페이스에 바인딩하지 않고, 업무 서버는 관리 콘솔에서 시작하기 전까지 정지 상태를 유지한다.

## API 원칙

- API는 `/api/v1`로 버전 관리하고 리소스 중심 URL과 HTTP method를 사용한다.
- Pydantic은 transport DTO를 검증하며 업무 규칙은 application service와 DB 제약에서 다시 검증한다.
- FastAPI router는 얇게 유지하고 transaction 경계는 application service 또는 unit of work가 소유한다.
- 오류는 안정된 코드, 사용자 메시지, field detail, correlation ID를 가진 공통 형식으로 응답한다.
- OpenAPI 문서에서 TypeScript client와 타입을 생성하고 실제 응답과의 계약을 CI에서 검사한다.
- 동시성, 추첨, 복구처럼 단순 CRUD가 아닌 작업은 명시적인 command endpoint와 작업 상태를 사용한다.

## Python 아키텍처 원칙

- 기능별 모듈 안에서 `api → application → domain/port → infrastructure` 방향을 지킨다.
- SQLAlchemy model을 API response로 직접 반환하지 않는다.
- Pydantic model을 핵심 업무 규칙의 유일한 표현으로 사용하지 않는다.
- repository는 SQLAlchemy query를 캡슐화하고 application service가 여러 repository transaction을 조정한다.
- FastAPI `Depends`는 조립과 request scope에만 사용하며 도메인 코드에 유출하지 않는다.

## 배포 및 확장

- Windows 기본 배포는 PyInstaller `onedir` 결과와 런타임 폴더를 묶은 ZIP이다.
- React production build는 패키지에 포함하고 운영 PC에 Node.js를 요구하지 않는다.
- SQLite DB, 설정, 백업, 로그, Excel 파일은 번들 외부의 명확한 디렉터리에 둔다.
- 중앙 서버가 필요해지면 같은 FastAPI application을 Docker로 실행하고 PostgreSQL adapter를 추가한다.
- Docker 배포는 프로토타입 필수 범위가 아니며 Windows 로컬 운영을 먼저 완성한다.

## 의도적으로 쓰지 않는 것

- Go/Fyne/Wails: `go-prototype-baseline-2026-07` 태그에서만 보존하고 활성 구현에는 유지하지 않는다.
- Electron: 호스트 제어면만을 위해 브라우저 엔진과 Node runtime을 번들하지 않는다.
- PyInstaller `onefile`: 초기 기본값으로 사용하지 않는다. 실제 Windows 검증 후 보조 산출물로만 검토한다.
- Django: 현재 목표는 명시적인 API와 React UI이며 Django admin 중심 구조를 필요로 하지 않는다.
- 전역 상태 라이브러리: TanStack Query와 React local state로 부족하다는 근거가 생길 때만 도입한다.
