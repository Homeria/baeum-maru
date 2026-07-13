# 품질 체크리스트

## Python 백엔드와 데이터

- [ ] `uv lock --check`, Ruff format/lint, mypy 통과
- [ ] `pytest` unit/integration/contract 전체 통과
- [ ] 빈 DB에서 Alembic upgrade, 기준 데이터, backup/restore 확인
- [ ] 회원/강좌/신청/추첨/출석/Excel 한 cycle integration test
- [ ] 다중 신청 저장이 원자적이며 DB 제약과 application rule이 모두 동작
- [ ] FK, unique, check, index, cascade/null 정책이 실제 workflow와 일치
- [ ] SQLite connection과 transaction이 request/task 종료 시 정리됨

## REST API

- [ ] `/api/v1` resource, method, status code, pagination/filter 규약 확인
- [ ] Pydantic validation과 공통 오류 response 확인
- [ ] OpenAPI snapshot과 생성 TypeScript client가 최신 상태
- [ ] 권한, CSRF, idempotency, 동시성 충돌 test 통과
- [ ] host control API가 loopback 이외의 경로에서 접근되지 않음

## React UI

- [ ] TypeScript typecheck, lint, unit test, production build 통과
- [ ] 회원 등록과 다중 강좌 신청의 성공/오류/충돌 상태 확인
- [ ] keyboard, focus, table/form의 mobile 및 desktop layout 확인
- [ ] SSE 수신 시 관련 데이터만 갱신되고 입력 중 form을 덮어쓰지 않음
- [ ] 한글, 날짜/시간, 전화번호 입력이 Windows browser에서 정상 동작

## 호스트 콘솔

- [ ] 실행 직후 업무 서버가 정지 상태임
- [ ] 시작/중지/재시작과 중복 command 방지 확인
- [ ] 실제 내부망 IPv4 주소와 접속 URL이 명확히 표시됨
- [ ] 접속 코드, 로그, 백업, 설정 기능 확인
- [ ] localhost 이외에서 host console route/API 접근이 거부됨
- [ ] 종료 중 진행 작업과 server shutdown 상태가 사용자에게 보임

## 보안

- [ ] 내부망 공유는 명시적 설정이며 기본 bind는 localhost
- [ ] HTTPS, `Secure` session cookie, CSRF, 로그인 실패 제한 적용
- [ ] 역할 권한이 API와 SSE 모두에서 강제됨
- [ ] file upload 크기/확장자/parsing 오류가 안전하게 처리됨
- [ ] 로그, backup, Excel, screenshot에 개인정보가 불필요하게 포함되지 않음

## 운영과 패키징

- [ ] 2~5개 browser 동시 접수 scenario 수행
- [ ] 회원 1,000명, 강좌 50개, 신청 3,000건 dummy data 점검
- [ ] 저사양에 가까운 Windows 사무용 노트북에서 package 실행
- [ ] Python과 Node.js가 설치되지 않은 Windows에서 실행
- [ ] 한글 사용자명·한글 경로·공백 경로에서 runtime directory 확인
- [ ] network 단절, server 재시작, SSE 재연결, backup/restore scenario 확인
- [ ] Windows Defender와 기관 보안 정책에서 실행/오류 흐름 확인
