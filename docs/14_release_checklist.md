# 릴리즈 체크리스트

## 코드와 테스트

- [ ] `go test ./...` 통과
- [ ] `go test -race ./...` 통과
- [ ] `go vet ./...` 통과
- [ ] `go build ./cmd/server` 통과
- [ ] Wails Windows 빌드 통과
- [ ] Huma OpenAPI spec 생성 및 API 호환성 검사 통과
- [ ] React typecheck, lint, unit test, production build 통과
- [ ] `git diff --check` 통과

## 패키지

- [ ] Windows 패키지에 Wails 런처, 콘솔 서버 fallback, 서버 자산, 기본 `config.json`, 런타임 폴더, 첫 실행 안내 포함
- [ ] WebView2 런타임이 없는 PC의 설치/오류 흐름 확인
- [ ] 오프라인 WebView2 설치 수단 또는 fallback 안내 포함
- [ ] 런처가 기본적으로 서버 정지 상태로 열리는지 확인
- [ ] 서버 시작 후 실제 내부망 주소와 HTTPS URL을 표시하는지 확인
- [ ] 패키지에서 Node.js나 개발 도구 설치를 요구하지 않는지 확인

## 운영 리허설

- [ ] 접속 코드 발급, 로그인, 만료/폐기 확인
- [ ] 회원 등록/검색과 다중 강좌 접수 확인
- [ ] 동시 접수, 충돌 메시지, SSE 재연결 확인
- [ ] 강좌 운영, 추첨, 확정/대기자 승격, 출석, Excel 출력 확인
- [ ] 백업 생성, 복구 예약, 재시작 후 복구 확인
- [ ] 서버 중지/재시작/종료와 로그 확인

## 보안과 공개

- [ ] HTTPS 인증서, Secure cookie, CSRF, 로그인 실패 제한 확인
- [ ] 실사용 DB, 백업, Excel, 로그, 인증서 비밀값이 패키지/저장소에 없음
- [ ] `LICENSE`, `NOTICE`, `CONTRIBUTING.md`, `SECURITY.md` 확인
- [ ] README와 사용자 가이드가 현재 런처/UI 구조를 설명함
- [ ] 예시 화면과 더미 데이터에 개인정보가 없음

## CI

- [ ] Pull request에서 Go와 frontend 검사가 실행됨
- [ ] Windows runner에서 Wails/WebView2 패키지 smoke test가 실행됨
- [ ] 태그 릴리즈에서 portable artifact 또는 설치 패키지가 생성됨
