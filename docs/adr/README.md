# 아키텍처 결정 기록 (ADR)

바꾸기 비싼 결정을 왜 그렇게 정했는지 남기는 문서다. 결정은 유지보수자가 하고, 결정한 것만 기록한다.

- 파일 이름: `번호-제목.md` (예: `0001-app-form.md`)
- 한 번 채택한 기록은 고치지 않는다. 결정을 바꾸면 새 기록을 쓰고, 옛 기록의 상태를 "대체됨 (00NN)"으로 바꾼다.
- 내용: 상태, 상황, 선택지, 결정, 이유, 결과

| 번호 | 결정 | 상태 |
|---|---|---|
| [0001](0001-app-form.md) | 모든 직원 PC에 같은 앱, 웹 접속은 예비 | 일부 대체됨 (0007) |
| [0002](0002-data-owner.md) | 호스트 PC 한 대가 DB를 갖고, 호스트는 파일로 넘긴다 | 채택 |
| [0003](0003-login-access.md) | 로그인: 기기 승인(요청형) + 직원 PIN | 채택 |
| [0004](0004-host-discovery.md) | 호스트 찾기: 주소 입력 + 기억, 포트 47800 고정, 자동 찾기는 나중 | 채택 |
| [0005](0005-storage.md) | 저장소: SQLite + sqlx, 구조 변경은 자동 적용 + 사전 백업 | 채택 |
| [0006](0006-crate-layout.md) | 코드 나누기: 업무 규칙 crate(`crates/domain`) + 앱 crate(`src-tauri`) | 채택 |
| [0007](0007-app-only-access.md) | 앱으로만 접속 (0001의 웹 예비 대체) | 채택 |
| [0008](0008-http-json-sse.md) | 화면과 서버의 대화: HTTP + JSON 요청, SSE 변경 알림 | 채택 |
| [0009](0009-frontend.md) | 화면 기술: React + Vite, 한 번에 그리는 줄 수 제한 | 채택 |
| [0010](0010-distribution.md) | 배포와 업데이트: 설치판 + 완전 포터블판, 출시는 서명 필수, 버전 번호로 호환 판단 | 채택 |
| [0011](0011-backup.md) | 백업과 복원: 자동 백업(하루·종료·작업 직전), 두 번째 위치, 통째 복원 | 채택 |
| [0012](0012-skeleton.md) | 뼈대: TypeScript·앱 내장 화면·TanStack Query·React Compiler·ts-rs, just check, GitHub Flow, CI 경로 제한, Dependabot·cargo-deny | 일부 대체됨 (0013, 0020) |
| [0013](0013-defer-dependabot.md) | Dependabot 버전 업데이트는 출시 준비 때 붙인다 (0012의 일부 대체) | 채택 |
| [0014](0014-staff-login.md) | 직원 확인 구현: argon2 PIN 해시, DB 세션, 잠금 값 | 채택 |
| [0015](0015-date-time-crate.md) | 날짜·시각은 time crate로 다룬다 (chrono 관리 중단) | 채택 |
| [0016](0016-lottery-random.md) | 추첨 난수: 멈춤 순간의 OS 난수 시드 + 직접 만든 SplitMix64·피셔-예이츠 섞기 | 채택 (0017에서 대상·자리 수 보완) |
| [0017](0017-priority-allocation.md) | 우선 배정: 기관이 만드는 사유 목록 + 사유마다 배정 방식(지금은 자리 보장), 신청마다 표시 | 채택 |
| [0018](0018-save-dialog.md) | 결과 명단 파일은 받는 PC에서 앱(Rust)이 저장 창을 띄워 저장한다 (tauri-plugin-dialog) | 채택 |
| [0019](0019-website.md) | 웹사이트는 이 저장소의 site/에 두고 GitHub Actions로 Pages에 올린다 (공개 배포는 출시 준비 때) | 채택 |
| [0020](0020-split-ci-jobs.md) | 앱 검사는 작업 셋(화면, 의존성, Rust)으로 나눠 함께 돌린다 (0012의 일부 대체) | 채택 |
