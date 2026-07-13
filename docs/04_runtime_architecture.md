# 배움마루 런타임 아키텍처

## 목표 구조

```text
호스트 Windows 노트북
├─ 배움마루.exe (PyInstaller onedir entry point)
│  └─ Python 프로세스
│     ├─ Host control server: 127.0.0.1 전용
│     │  ├─ 호스트 React 콘솔 제공
│     │  ├─ 업무 서버 lifecycle과 네트워크 설정
│     │  └─ 접속 코드, 로그, 백업, 초기 기준 데이터
│     └─ Operator server: 설정된 host:port
│        ├─ FastAPI /api/v1와 OpenAPI
│        ├─ /api/v1/events SSE
│        ├─ 직원 React 앱 제공
│        └─ 인증, 권한, CSRF, HTTPS
├─ SQLite: data/center.db
└─ config / backups / exports / imports / logs

직원 브라우저
└─ https://호스트-IP:포트
   └─ React 직원 앱
```

## 프로세스와 제어면

- 실행 파일이 시작되면 localhost 전용 호스트 제어면만 먼저 연다.
- 업무 서버는 기본적으로 정지 상태이며 담당자의 명시적인 시작 동작을 요구한다.
- 호스트 제어면은 LAN 주소에 바인딩하지 않는다.
- 업무 서버는 설정된 bind host와 port로 별도 Uvicorn server task를 실행한다.
- 시작, 중지, 재시작은 상태 머신과 timeout을 가지며 중복 명령을 거부한다.
- 애플리케이션 종료는 진행 중 작업, DB connection, 업무 서버를 순서대로 정리한다.

## 실행 흐름

1. 담당자가 포터블 폴더의 `baeum-maru.exe`를 실행한다.
2. 애플리케이션은 설정, 디렉터리, DB, migration, pending restore를 검증한다.
3. 기본 브라우저에서 localhost 호스트 콘솔을 연다.
4. 담당자는 bind mode, port, 접속 코드, 기관 설정을 확인하고 업무 서버를 시작한다.
5. 콘솔은 실제 내부망 IPv4와 직원 접속 URL을 보여준다.
6. 직원은 브라우저에서 접속 코드로 로그인한다.
7. React 앱은 REST API로 데이터를 변경하고 SSE로 관련 domain 변경을 수신한다.
8. 종료 시 업무 서버와 DB를 graceful shutdown하고 로그를 남긴다.

## 동기화 원칙

- WebSocket 대신 SSE를 사용한다. 서버에서 브라우저로 변경 사실을 알리는 단방향 흐름이 중심이다.
- 이벤트는 `members`, `courses`, `registrations`, `lottery`, `attendance`, `settings` 같은 scope를 가진다.
- React는 수신한 scope의 TanStack Query cache를 무효화하고 필요한 데이터만 다시 읽는다.
- 폼 편집 중에는 자동 덮어쓰지 않고 최신 데이터가 있다는 안내를 표시한다.
- 최종 정확성은 SSE가 아니라 DB 제약, transaction, idempotency로 보장한다.

## 네트워크와 보안

- 호스트 제어면은 항상 loopback 전용이다.
- 업무 서버 기본 bind는 `127.0.0.1`이며 담당자가 내부망 공유를 명시적으로 켜야 LAN에 연다.
- 내부망 공유 시 HTTPS를 사용하고 자체 서명 인증서의 신뢰 절차를 사용자 가이드에 기록한다.
- cookie는 `HttpOnly`, `SameSite`, HTTPS 사용 시 `Secure`를 적용한다.
- 상태 변경 요청은 CSRF token을 검증한다.
- 접속 코드 로그인은 만료, 폐기, 역할, 실패 횟수 제한을 적용한다.
- 프록시 header는 기본적으로 신뢰하지 않으며 실제 client address를 권한 판단에 사용하지 않는다.

## 저장과 복구

- DB와 운영 파일은 PyInstaller bundle과 분리된 writable directory에 둔다.
- SQLite는 WAL, foreign key, busy timeout을 적용한다.
- 복구는 실행 중 DB 파일을 덮어쓰지 않고 다음 안전한 재기동 시점에 적용한다.
- 복구 직전 현재 DB를 별도 백업하고 성공 여부를 감사 로그에 남긴다.

## 서버 확장

중앙 서버가 필요해지면 operator FastAPI application을 Docker에서 실행한다. 이때 localhost 호스트 제어면은 배포 대상에서 제외할 수 있고, PostgreSQL repository와 외부 reverse proxy/인증서 구성을 추가한다. 로컬 SQLite와 중앙 서버 배포는 같은 OpenAPI와 업무 규칙 테스트를 공유한다.
