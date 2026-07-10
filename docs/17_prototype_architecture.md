# 프로토타입 아키텍처 결정

## 상태

채택 예정. 현재 Go template/Fyne 구현을 대체하기 전의 목표 구조다.

## 결정

배움마루의 프로토타입은 다음처럼 구성한다.

```text
Go + net/http + Huma v2 + SQLite
├─ React/Vite/TypeScript 웹 앱: 직원과 관리자 브라우저 사용
├─ SSE: 도메인 변경 알림
└─ Wails v2 + React 런처: 호스트 노트북 운영 콘솔
```

## 이유

- 사용자 화면은 여러 명이 같은 데이터를 다루므로 브라우저/서버 구조가 가장 자연스럽다.
- 접수, 검색, 필터, 다중 선택, 모달, 실시간 상태는 컴포넌트 기반 UI가 유리하다.
- 런처는 서버 제어와 운영 설정을 위한 전용 Windows 앱이며, Fyne보다 WebView2 기반 UI가 한글 입력과 복잡한 화면에 적합하다.
- 기존 Go 서비스/DB 코드를 유지해 Rust/Tauri 재작성 비용을 피한다.
- Windows 전용 환경에서는 WebView2 런타임 의존성을 현실적으로 관리할 수 있다.
- Huma v2는 표준 `net/http` 위에서 OpenAPI와 검증을 제공하므로, API 계약을 분명히 하면서 프레임워크 종속성을 transport 계층에 가둔다.

## 하지 않는 것

- Wails를 직원용 클라이언트로 배포하지 않는다.
- Wails 때문에 외부 웹페이지나 인터넷 연결을 요구하지 않는다.
- React를 곧바로 두 앱의 모든 컴포넌트에 공용화하지 않는다. 실제로 공유되는 디자인 토큰과 안정된 컴포넌트만 `packages/ui`로 승격한다.
- 현재 Fyne/template 구현을 검증 없이 삭제하지 않는다.

## 호환성 조건

- 운영 PC에는 Node.js, Python, Docker Desktop, C 컴파일러를 요구하지 않는다.
- React/Vite 산출물과 Go SQL migration은 실행 파일에 포함한다.
- WebView2가 없거나 설치 정책이 막힌 경우에도 콘솔 서버 fallback으로 브라우저 업무를 계속한다.
- 패키지와 CI는 Windows에서 실제 빌드/실행을 확인한다. 단일 exe 크기보다 오류 없이 업무를 이어갈 수 있는 portable ZIP을 우선한다.

## 전환 순서

1. 프레임워크 독립 런처 서비스와 JSON API 경계를 만든다.
2. Wails 최소 프로토타입으로 Windows/WebView2/한글 입력을 검증한다.
3. React 웹 기반과 핵심 접수 흐름을 구현한다.
4. Wails 런처를 전환한다.
5. 이전 Fyne/template 경로를 제거하거나 유지보수 모드로 명확히 구분한다.
