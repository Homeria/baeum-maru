# 배움마루 기술 스택

이 문서는 프로토타입에서 반드시 따를 채택 기술 스택이다. 변경은 실제 검증 결과가 현재 조건을 더 잘 만족하지 못한다는 근거가 있을 때만 ADR과 함께 진행한다.

## 결정

| 영역 | 선택 | 역할 |
|---|---|---|
| 백엔드 | Go 1.25.x, 표준 라이브러리 `net/http` | HTTP API, 업무 규칙, 파일/서버 제어 |
| API 계약 | Huma v2, OpenAPI 3.1 | 타입 기반 요청/응답 검증, 문서, 표준 오류 응답 |
| DB | SQLite, `database/sql`, `modernc.org/sqlite` | 호스트 로컬 데이터 저장 |
| 웹 UI | React, TypeScript, Vite | 직원/관리자 브라우저 화면 |
| 웹 상태 | TanStack Query, SSE | 서버 데이터 조회, 변경 이벤트 기반 갱신 |
| 폼/검증 | React Hook Form, Zod | 접수와 강좌 운영의 복잡한 입력 검증 |
| 스타일 | CSS variables와 CSS Modules, Lucide icons | 기관 업무 도구에 맞는 일관된 UI |
| 호스트 런처 | Wails v2, React/Vite, Windows WebView2 | 서버 제어와 운영 콘솔 |
| Excel | Excelize | Excel 가져오기/내보내기 |
| 설정/로그 | JSON, `log/slog` | 포터블 설정과 구조화 로그 |
| 검증 | Go test, race detector, Vitest, Playwright | 도메인, UI, 다중 브라우저 흐름 검증 |
| CI/CD | GitHub Actions | Go/API/UI 검사와 Windows 패키징 |

## 선택 원칙

- Windows 사무용 노트북과 내부망 5명 내외 사용을 기준으로 한다.
- 사용자용 앱을 설치하지 않는다. 브라우저가 직원 클라이언트다.
- 런처와 웹은 UI 기술을 공유하지만, 업무 서비스와 DB 접근은 Go에 둔다.
- Node.js는 개발/빌드에만 사용한다. 최종 웹 자산은 Go 실행 파일에 포함되며, 운영 PC에서 Node.js를 설치할 필요는 없다.
- Electron처럼 브라우저 엔진을 번들하지 않는다. Wails는 Windows의 WebView2 런타임을 사용한다.
- 운영 PC는 Go, Node.js, Python, Docker Desktop, C 컴파일러를 설치하지 않아도 된다.
- 모든 의존성은 버전을 잠그고 Windows CI에서 실제 패키지 빌드를 검증한다.

## 런처와 웹의 역할

```text
Wails 런처 (호스트 PC만)
  └ Go 런처 서비스 직접 호출
      └ Go HTTP 서버 시작
          └ React 웹 자산 제공
              └ 직원 브라우저가 내부망으로 접속
```

Wails는 외부 웹사이트를 필요로 하지 않는다. 런처의 React 빌드 산출물을 실행 파일에 포함해 창 내부 WebView2에서 그린다. 직원용 React 웹은 Go 서버가 정적 자산으로 제공한다.

## 현재 구현과 전환

현재 코드는 Go HTML template와 Fyne를 사용한다. 이는 업무 규칙과 운영 흐름을 검증한 기존 구현이며, 목표 스택으로 전환할 때 Go `service`와 `repository`는 유지한다.

- Go template handler: React 전환 기간의 호환/검증용으로 유지 후 제거한다.
- Fyne launcher: Wails 전환 완료 전까지 호스트 운영 기능을 유지한다.
- 새 React 화면은 template POST 경로를 호출하지 않고 Huma v2 기반 `/api/v1` JSON API를 사용한다.
- Wails 런처는 HTTP API를 우회해 프레임워크 독립 `internal/launcher` 서비스를 호출한다.

## API 원칙

- `net/http`가 transport 기반이고 Huma v2가 OpenAPI/검증 계층이다. 도메인 서비스는 Huma 타입을 import하지 않는다.
- API는 `/api/v1`로 버전 관리하고, 리소스 중심 URL과 HTTP method를 사용한다.
- 접수 처리와 추첨 실행처럼 원자적 업무 명령은 `reception-submissions`, `lottery-runs` 같은 리소스 생성으로 표현한다.
- 요청/응답 DTO, 오류 코드, 권한 규칙은 OpenAPI에 문서화하고 TypeScript client를 생성한다.
- API 호환성을 깨는 변경은 새 version 또는 명시적 deprecation 기간을 거친다.

## 포터블 호환성 정책

- Wails 런처는 Windows WebView2를 사용한다. 패키지는 런타임 부재 감지와 설치 경로를 제공한다.
- 인터넷이 없거나 WebView2 설치가 막힌 PC에서도 업무를 계속할 수 있도록 콘솔 서버 실행 경로를 패키지에 유지한다.
- 기본 배포 단위는 단일 exe만을 고집하지 않는 portable ZIP이다. ZIP에는 런처, fallback 서버, 설정, 런타임 폴더, WebView2 오프라인 설치 수단, 첫 실행 안내를 포함한다.
- WebView2 bootstrapper는 편의 수단이며, 기관 네트워크가 차단된 경우를 대비해 오프라인 설치 수단을 함께 검증한다.

## 의도적으로 쓰지 않는 것

- Rust/Tauri: 현재 Go 도메인 코드를 재작성해야 하므로 선택하지 않는다.
- Electron: 런처 용도에 비해 배포 크기와 런타임이 크다.
- Gin/Fiber: 현재 `net/http` 기반 코드와 API 계약 계층을 교체할 만큼의 이득이 없다.
- 전역 상태 라이브러리: TanStack Query와 React local state로 부족해질 때만 도입한다.
- UI 컴포넌트 프레임워크: 기관 업무 화면의 밀도와 디자인을 먼저 정하고, 필요한 primitive만 도입한다.
