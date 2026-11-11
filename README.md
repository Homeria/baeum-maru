# 배움마루

복지관·문화센터·평생교육기관의 수강신청, 추첨, 대기자, 엑셀 출력 업무를 **내부망에서만** 처리하는 포터블 업무 도구.

- 호스트 노트북 한 대가 Tauri 런처로 업무 서버를 켜고, 직원 최대 5명이 브라우저로 접속한다.
- 회원 개인정보를 다루므로 클라우드를 쓰지 않고, 데이터는 호스트의 SQLite 파일에만 둔다.

## 구조

```text
crates/
  domain/     순수 업무 규칙 (IO·async 금지)
  storage/    SQLite, 마이그레이션, repository
  app/        유스케이스, 트랜잭션, commit 후 이벤트
  api/        axum HTTP/WebSocket, 인증, OpenAPI
  launcher/   Tauri 호스트 런처
xtask/        코드 생성·패키징 개발 도구
frontend/     pnpm workspace (operator, launcher, api-client)
docs/         GLOSSARY · BUSINESS_RULES · adr/
```

의존 방향은 `domain ← storage ← app ← api ← launcher` 하나뿐이며 crate 경계가 이를 강제한다.

## 개발

```bash
just          # 레시피 목록
just check    # 빠른 컴파일 검사
just gen      # OpenAPI, TS client, sqlx 메타데이터 생성
just ci       # PR 전 전체 검증 (CI와 동일)
```

필요 도구: rustup, just, cargo-nextest, cargo-deny, sqlx-cli, pnpm.

## AI 개발 환경

이 저장소는 Claude Code를 포함한 코딩 에이전트와 함께 개발한다.

| 파일 | 역할 | 강제력 |
|---|---|---|
| `AGENTS.md` | 모든 에이전트 공통 규칙 원본 | 권고 |
| `CLAUDE.md` | `@AGENTS.md` import + Claude 전용 메모 | 권고 |
| `.claude/rules/` | 경로별 규칙 (해당 파일 작업 시에만 로드) | 권고 |
| `.claude/settings.json` | 권한(개인정보 열람 차단 등), 훅, 플러그인 | **강제** |
| `.claude/hooks/` | 생성 파일·머지된 마이그레이션 수정 차단, 포맷, 턴 종료 시 컴파일 검사 | **강제** |
| `.claude/skills/` | 반복 절차 (`/slice`, `/migration`, `/business-rule`, `/release-check`) | 절차 |
| `.claude/agents/` | 리뷰 전용 서브에이전트 (`rust-reviewer`, `privacy-reviewer`) | 리뷰 |
| `.github/workflows/ci.yml` | 최종 관문 (`just ci`) | **강제** |

원칙: 지켜져야만 하는 규칙은 문서가 아니라 lint·훅·권한·CI로 강제한다. 문서 규칙이 반복해서 무시되면 훅이나 lint로 옮긴다.

개인 설정은 `.claude/settings.local.json`, `CLAUDE.local.md`에 두며 커밋하지 않는다.

## 데이터 주의

`runtime/`(실제 DB, 백업, 출력물, 로그)은 Git에 올리지 않고, 코딩 에이전트도 읽을 수 없도록 차단되어 있다. 개발과 테스트에는 `testdata/`의 가짜 데이터만 사용한다.

## 라이선스

MIT
