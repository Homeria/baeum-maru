# 배움마루 런타임 아키텍처

## 목표 구조

```text
호스트 Windows 노트북
├─ 배움마루 런처.exe (Wails + WebView2)
│  ├─ 서버 제어, 설정, 접속 코드, 로그, 백업
│  └─ internal/launcher 서비스 호출
├─ Go HTTP 서버
│  ├─ Huma v2 기반 /api/v1 JSON API와 OpenAPI 문서
│  ├─ /api/v1/events SSE
│  ├─ React 웹 자산 제공
│  └─ 인증/권한/CSRF/HTTPS 처리
└─ SQLite: data/center.db

직원 브라우저
└─ https://호스트-IP:포트
    └─ React 웹 앱
```

## 실행 흐름

1. 담당자가 런처를 실행한다.
2. 런처는 설정과 DB 상태를 읽고 서버를 정지 상태로 보여준다.
3. 담당자가 서버 시작을 누르면 런타임 디렉터리, DB, 마이그레이션, HTTPS 설정을 확인한 뒤 서버를 시작한다.
4. 런처는 실제 내부망 IPv4 주소와 접속 URL을 목록으로 보여주고 복사를 지원한다.
5. 직원은 브라우저에서 접속 코드로 로그인한다.
6. React 웹은 JSON API로 데이터를 읽고 쓰며, SSE로 다른 사용자의 변경을 수신한다.
7. 서버 중지/재시작/종료는 graceful shutdown과 작업 상태 확인을 거친다.

## 동기화 원칙

- WebSocket 대신 SSE를 사용한다. 이 시스템은 서버에서 클라이언트로 변경 사실을 알리는 단방향 흐름이 중심이다.
- 이벤트는 `members`, `courses`, `registrations`, `lottery`, `attendance`, `settings` 같은 도메인 scope를 가진다.
- React는 수신한 scope의 query cache를 무효화하고 필요한 목록만 다시 읽는다.
- 사용자가 폼을 편집 중이면 자동 덮어쓰지 않고, 최신 데이터가 있다는 안내를 표시한다.
- 최종 정확성은 이벤트가 아니라 DB 제약과 트랜잭션으로 보장한다.

## 네트워크와 보안

- 기본 바인딩은 `127.0.0.1`이다. 담당자가 내부망 공유를 명시적으로 켜야 `0.0.0.0`에 바인딩한다.
- 내부망 공유 시 HTTPS를 사용한다. 자체 서명 인증서의 최초 신뢰 절차와 인증서 교체 방법을 사용자 가이드에 적는다.
- 쿠키는 `HttpOnly`, `SameSite`, HTTPS 사용 시 `Secure`를 적용한다.
- POST/PUT/PATCH/DELETE 요청은 CSRF 토큰을 검증한다.
- 접속 코드 로그인은 만료, 폐기, 역할, 실패 횟수 제한을 적용한다.

## 현재 구현과 차이

현재는 Go template 경로와 `/events` SSE, Fyne 런처가 동작한다. 목표 구조의 Huma `/api/v1`, React 자산, Wails 런처, HTTPS/CSRF는 전환 작업으로 구현한다.

## 호환성 fallback

Wails 런처가 WebView2 부재나 기관 PC 정책으로 실행되지 못해도 Go 서버와 브라우저 업무가 멈추면 안 된다. 포터블 패키지는 콘솔 서버 실행 경로를 함께 제공한다. 이 경로는 관리자/직원이 브라우저에서 최소 업무를 계속할 수 있는 복구 수단이며, Wails 전환 후에도 한 프로토타입 주기 동안 유지한다.
