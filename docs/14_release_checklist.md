# 릴리즈 체크리스트

## 코드와 계약

- [ ] `uv lock --check` 통과
- [ ] Ruff format/lint와 mypy 통과
- [ ] pytest unit/integration/contract test 통과
- [ ] Alembic empty DB upgrade test 통과
- [ ] OpenAPI spec 생성과 TypeScript client diff 검사 통과
- [ ] React typecheck, lint, unit test, production build 통과
- [ ] Playwright 핵심 workflow 통과
- [ ] `git diff --check` 통과

## Windows 패키지

- [ ] PyInstaller `onedir` 결과가 portable ZIP에 포함됨
- [ ] 실행 파일, React assets, 기본 config, runtime directory, 첫 실행 안내 포함
- [ ] DB, backup, export, import, log가 bundle 외부 writable path에 생성됨
- [ ] Python, Node.js, uv 설치를 요구하지 않음
- [ ] 한글 사용자명, 한글/공백 경로에서 실행됨
- [ ] 실행 직후 업무 서버는 정지 상태이고 pywebview 독립 런처 창만 열림
- [ ] WebView2 Runtime 유무를 검사하고 누락 시 설치/복구 안내를 제공함
- [ ] 업무 서버 시작 후 실제 내부망 주소와 HTTPS URL을 표시함
- [ ] 비정상 종료 후 재실행과 오류 log 확인 가능

## 운영 리허설

- [ ] 접속 코드 발급, 로그인, 만료/폐기 확인
- [ ] 회원 등록/검색과 다중 강좌 접수 확인
- [ ] 동시 접수, conflict message, WebSocket 재연결 확인
- [ ] 강좌 운영, 추첨, 확정/대기자 승격, 출석, Excel 출력 확인
- [ ] backup 생성, restore 예약, 재시작 후 복구 확인
- [ ] server 시작/중지/재시작/종료와 log 확인

## 보안과 공개

- [ ] launcher bridge와 권한 있는 명령이 LAN에 공개되지 않음
- [ ] HTTPS 인증서, Secure cookie, CSRF, 로그인 실패 제한 확인
- [ ] 실사용 DB, backup, Excel, log, 인증서 secret이 package/source에 없음
- [ ] `LICENSE`, `NOTICE`, `CONTRIBUTING.md`, `SECURITY.md` 확인
- [ ] README와 사용자 가이드가 현재 Python 구조를 설명함
- [ ] example과 screenshot에 개인정보가 없음

## CI와 릴리즈

- [ ] PR에서 Python backend, OpenAPI, frontend 검사가 실행됨
- [ ] Windows runner에서 PyInstaller build와 portable smoke test가 실행됨
- [ ] tag release에서 portable ZIP artifact가 생성됨
- [ ] release artifact checksum과 version 정보가 제공됨
