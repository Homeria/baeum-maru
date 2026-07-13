# 프로토타입 아키텍처 결정

## 상태

채택. 2026-07-13부터 Python/FastAPI 구현을 활성 기준으로 사용한다.

## 결정

```text
Python 3.13 + FastAPI + Pydantic v2
├─ SQLAlchemy 2 + Alembic + SQLite WAL
├─ React/Vite/TypeScript 직원 업무 앱
├─ pywebview/WebView2 + React/Vite/TypeScript 독립 런처
├─ REST /api/v1 + OpenAPI + WebSocket
└─ PyInstaller onedir Windows portable ZIP
```

기존 Go 구현은 `go-prototype-baseline-2026-07` annotated tag에 보존하고 활성 tree에서 제거한다.

## 변경 이유

- 프로젝트 소유자가 FastAPI 경험을 바탕으로 코드를 직접 읽고 검증할 수 있다.
- Pydantic, FastAPI dependency, OpenAPI를 이용해 API 경계를 명시적으로 유지할 수 있다.
- Python의 구현 속도는 반복되는 현장 피드백과 schema/application rule 수정에 적합하다.
- 목표 동시 사용자는 2~5명이며 Python runtime overhead는 핵심 제약이 아니다.
- 실사용·배포 데이터가 없어 언어 전환과 schema 초기화 비용이 낮다.
- 향후 중앙 서버가 필요하면 FastAPI application을 Docker로 배포하기 쉽다.
- 비상업적 포크 프로젝트에서 Python은 기여자와 기관 개발자가 접근하기 쉽다.

## 수용하는 비용

- Go binary보다 package 크기와 memory 사용량이 증가한다.
- native single binary가 아니라 Python interpreter와 dependency를 포함한 directory bundle을 배포한다.
- startup과 antivirus 호환성은 실제 Windows artifact에서 반복 검증해야 한다.
- Python의 dynamic 특성을 보완하기 위해 mypy, Ruff, Pydantic, DB constraint, pytest를 함께 사용한다.

이 비용은 프로젝트 소유자의 검증 가능성, 개발 속도, 포크 접근성이 주는 이점보다 작다고 판단한다.

## 호스트 런처 결정

별도 native launcher를 사용한다. 실행 파일은 pywebview로 WebView2 독립 창을 열고 bundled React launcher app을 표시한다.

- launcher는 server lifecycle, network, access code, log, backup, 초기 설정을 담당한다.
- operator server는 별도 Uvicorn 자식 프로세스이며 기본 상태는 stopped다.
- React launcher는 검증된 Python bridge를 호출하고 권한 있는 control endpoint를 LAN socket에 노출하지 않는다.
- launcher는 bundled local assets만 로드하고 production에서는 원격 navigation과 개발자 도구를 차단한다.
- WebView2 Runtime 유무를 시작 시 검사하고 누락 시 Microsoft Evergreen 설치 수단을 제공한다.

## application 아키텍처

stateless modular monolith와 기능 우선 layered architecture를 사용한다.

```text
api adapter (FastAPI/Pydantic)
          ↓
application use case + unit of work
          ↓
domain rule + repository protocol
          ↑
SQLAlchemy / filesystem / Excel adapter
```

- 루트 계층별 디렉터리보다 기능 모듈 안에서 router, schema, service, domain, repository, model을 함께 관리한다.
- 단순 CRUD에 불필요한 abstraction을 강제하지 않는다.
- 여러 table과 repository를 바꾸는 업무 command는 application transaction으로 묶는다.
- FastAPI와 SQLAlchemy model을 업무 규칙 자체로 사용하지 않는다.
- domain event는 audit log와 commit 이후 WebSocket 발행을 HTTP handler 밖에서 연결한다.
- module 간 호출은 public application interface를 통해 수행한다.
- 변경 command는 application service와 unit of work를 사용하고 단순 목록/검색 query는 읽기 전용 projection을 허용한다.
- generic repository, full CQRS framework, event sourcing은 도입하지 않는다.

## stateless 원칙

- FastAPI 프로세스 메모리에 업무 데이터와 로그인 session을 보관하지 않는다.
- SQLite를 업무 상태의 source of truth로 사용하고 서버 cache는 두지 않는다.
- session, operation job, lottery lock, idempotency key는 DB에 저장한다.
- WebSocket connection registry는 일시적인 transport 상태일 뿐이며 재연결 시 REST API로 확정 데이터를 다시 읽는다.
- pywebview launcher의 child process 상태는 업무 서버와 분리한다.
- FastAPI 재시작 후 DB와 설정만으로 모든 업무 상태를 복구할 수 있어야 한다.

TanStack Query의 browser cache는 서버 cache와 구분한다. client cache는 WebSocket event 수신과 재연결 시 무효화하고 REST API 응답으로 재구성한다.

## 데이터 결정

- 과거 Go DB를 변환하는 migration은 작성하지 않는다.
- 최신 schema 문서와 `001_init.sql`의 의미를 검토해 단일 Alembic initial revision을 새로 작성한다.
- initial revision 이후부터 모든 schema 변경을 Alembic history로 관리한다.
- SQLite constraint와 transaction test를 source of truth로 사용한다.
- 중앙 서버 확장 시 PostgreSQL adapter를 추가하되 domain/API contract를 유지한다.

## 배포 결정

- 기본 artifact는 PyInstaller `onedir` directory를 ZIP으로 묶은 portable package다.
- `onefile`은 압축 해제 startup과 임시 directory 문제 때문에 기본값으로 사용하지 않는다.
- frontend production assets와 Python runtime을 package에 포함한다.
- pywebview dependency와 launcher production assets를 포함하고 WebView2 Runtime 검사 흐름을 검증한다.
- DB, config, certificate, backup, export, log는 bundle 밖에 생성한다.
- 운영 PC는 Python, uv, Node.js를 설치하지 않는다.
- Windows CI와 실제 사무용 노트북 smoke test를 모두 통과해야 release 후보가 된다.

## 폐기한 대안

- Go + Huma: 배포 효율은 좋지만 프로젝트 소유자의 코드 검증 비용이 크다.
- Fyne: 한글 IME와 복잡한 UI 경험이 목표에 맞지 않았다.
- Wails: Go backend가 전제이며 Python 전환 후 유지할 이유가 없다.
- Electron: 호스트 런처 용도에 비해 runtime과 package가 무겁다.
- CustomTkinter: 단순성과 외부 WebView 의존성은 유리하지만 React 디자인 체계 재사용과 복잡한 런처 UI 확장성을 우선해 선택하지 않는다.
- Python과 Go 병행: 실사용 호환성 요구가 없고 이중 유지보수만 만든다.
- 새 repository: 제품 정체성, issue, license, schema history가 같으므로 기존 repository를 유지한다.

## 재검토 조건

다음 중 하나가 실제 측정으로 확인될 때 이 결정을 다시 검토한다.

- 목표 Windows PC에서 PyInstaller package가 반복적으로 실행되지 않는다.
- idle/업무 memory 또는 startup이 운영을 방해한다.
- SQLite와 단일 Python process가 2~5명 동시 업무를 감당하지 못한다.
- 중앙 서버 요구가 생기고 Python 운영 비용이 실제 장애 원인이 된다.

재검토 시에도 OpenAPI, schema, 업무 rule, integration scenario를 language-independent contract로 유지한다.
