# 파일 구조

## 현재 구조

```text
cmd/
  launcher/                 Fyne/콘솔 런처 진입점
  server/                   HTTP 서버 진입점
internal/
  app/                      런타임 조립
  config/ database/ migration/ logging/
  domain/ repository/ service/
  web/                      Go template handler와 SSE
web/
  templates/ static/        현재 embed되는 HTML/CSS/JS
docs/ scripts/ .github/
```

## 목표 구조

```text
cmd/
  launcher/                 Wails 런처 진입점
  server/                   독립 서버 실행 진입점
internal/
  app/                      런타임 조립
  launcher/                 UI 프레임워크와 무관한 서버/설정/진단 제어
  domain/ repository/ service/
  web/
    api/                    Huma v2 /api/v1 handler, request/response DTO, OpenAPI
    auth/                   세션, 권한, CSRF, 로그인 제한
    realtime/               SSE event 발행과 구독
    assets/                 React 웹 dist 제공
frontend/
  web/                      직원/관리자 React 앱
  launcher/                 Wails용 React 앱
  packages/ui/              검증된 공통 컴포넌트와 디자인 토큰
  packages/api-client/      브라우저 API 클라이언트와 타입
web-legacy/                 전환 중인 Go template 자산, 전환 종료 후 제거
scripts/                    Windows 패키징, 인증서/개발 도구 보조
```

## 경계 규칙

- `domain`, `repository`, `service`는 React, Wails, `net/http`를 import하지 않는다.
- `internal/launcher`는 Fyne/Wails 타입을 모르고 Go 서비스와 서버 lifecycle만 다룬다.
- `internal/web/api`는 서비스 호출 결과를 HTTP DTO로 변환한다.
- `internal/web/api`만 Huma v2를 import하며, 도메인 코드와 저장소 코드는 표준 Go 타입만 사용한다.
- React 컴포넌트는 API client 또는 Wails binding만 알고 SQL/도메인 저장소에 직접 접근하지 않는다.
- `packages/ui`는 두 앱에서 실제로 두 번 이상 쓰이는 안정된 컴포넌트만 옮긴다.
