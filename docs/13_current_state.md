# 현재 구현 상태

## 기준 시점

2026-07-10 코드 기준. 기능 MVP의 구현 기준선은 `develop` 브랜치다.

## 구현된 기반

- Go 1.25, SQLite/WAL/foreign key/busy timeout, SQL migration
- `domain → repository → service → web` 계층과 실제 SQLite 기반 서비스 테스트
- 회원, 회차, 강좌, 시간표, 장소, 신청, 추첨, 출석, Excel, 백업/복구, 감사 로그
- 접속 코드 발급/만료/폐기, 서명된 세션 쿠키, 역할 기반 권한
- SSE `/events`와 도메인 scope 기반 변경 이벤트
- Fyne 런처: 서버 제어, 내부망 주소, 접속 코드, 로그, 공간/건물/역할, 설정/백업 화면
- Go template 기반 웹: 회원, 강좌, 접수, 신청 현황, 추첨, 출석, 가져오기/내보내기, 설정
- GitHub Actions Go CI와 Windows portable ZIP workflow

## 최근 검증 결과

- `go test ./...` 통과
- `go test -race ./...` 통과
- `go vet ./...` 통과
- `go build ./cmd/server` 통과
- `go build -tags fyne ./cmd/launcher` 통과

## 현재 한계

- 웹은 Go template/폼 POST 중심이며 React API 구조가 없다.
- 접수는 기존 회원 선택 후 강좌 하나씩 저장한다. 회원 등록과 다중 강좌 선택을 한 작업으로 처리하지 않는다.
- 강좌 입력 UI가 정규화된 회차/과목/공간/복수 시간표 모델을 충분히 드러내지 못한다.
- Fyne 런처의 서버 제어 상태가 UI 프레임워크에 결합돼 있다.
- SSE는 기반이 있으나 React query cache와 연결되지 않았고 일부 화면은 전체 새로고침에 의존한다.
- HTTPS, CSRF, 로그인 실패 제한, 다중 브라우저 E2E/부하 검증이 없다.
- Huma API/OpenAPI, Fyne/Wails 빌드, 브라우저 UI 테스트를 CI에서 아직 수행하지 않는다.

## 다음 기준선

기능 MVP는 대부분 완료다. 다음 목표는 `Go net/http + Huma REST API + React/Vite 웹 + Wails 런처 + WebView2 fallback + 실제 운영 리허설`로 이루어진 프로토타입이다. 자세한 순서는 `06_development_plan.md`를 따른다.
