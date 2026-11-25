# 0023. 코드 나누기: app/ 아래 런처·서버·업무 규칙·화면, 서버 안은 기능별 폴더와 controller-service-repository

- 상태: 채택 (2026-11-25)
- 근거: [0002](0002-data-owner.md), [0006](0006-crate-layout.md), [0008](0008-http-json-sse.md), [0019](0019-website.md), [0021](0021-run-mode.md), `docs/conventions.md` "모듈 파일 배치"
- 대체: 0006의 "앱 crate 하나에 나머지 전부"

## 상황

0006에서 업무 규칙 crate(`crates/domain`)와 앱 crate(`src-tauri`) 둘로 나눴다. 호스트 넘기기(#82)까지 만들고 보니 앱 crate가 커졌다.

- DB, HTTP 서버, 백업, Excel, 실행 모드, 창·대화상자 명령이 한 crate의 모듈로 섞여 있다. Tauri를 쓰는 곳은 창·대화상자·다시 켜기뿐이다.
- 기능 하나의 코드가 층마다 흩어져 있다(예: `server/members.rs` + `db/members.rs`). 화면도 같다(`api/members.ts` + `features/members/`). 고칠 때 여러 폴더를 오간다.
- 모듈 사이에 순환이 있다. `db`는 구조 변경 직전 백업 때문에 `backup`을 부르고, `backup`의 복원은 `db`를 부른다. 한 crate 안의 순환은 컴파일러가 막지 않는다.
- 런처 쪽 명령(넘기기, 넘겨받기, 깨진 DB 복원)이 서버의 PIN 확인·오류 타입 같은 안쪽을 직접 쓴다. 같은 오류 문장이 두 곳에 적히기도 했다.
- 한 요청이 여러 테이블을 한 트랜잭션으로 다루는 곳(신청, 추첨, 선발 결과, 넘기기)이 생겼다. conventions는 이때 "순서를 엮는 층"을 둘지 정하라고 했다.
- 저장소 맨 위에 앱 코드(`src-tauri`, `crates`, `ui`, Rust 설정 파일)와 웹사이트(`site`)가 섞여 있다. `src-tauri`라는 이름으로는 하는 일이 보이지 않는다.

유지보수자는 응집도가 높고 의존성이 낮은, 순환 없는 구조를 원한다. controller-service-repository 구조에 가장 익숙하다.

## 선택지

**맨 위 폴더**
1. 지금처럼 앱 코드와 `site/`를 맨 위에 나란히
2. `app/`(앱을 이루는 넷)과 `site/`로 나눔

**crate 경계**
1. 지금처럼 둘
2. 셋: 업무 규칙, 서버(Tauri 모름), 런처(Tauri)
3. 더 잘게(DB, HTTP, 백업을 각각)

**런처 폴더 이름**: `launcher` / `desktop` / `src-tauri` 그대로

**서버 안의 배치**
1. 층별(가로): `db/`, `server/`
2. 기능별(세로) 폴더, 폴더 안에서 controller-service-repository
3. 기능별 폴더, 층을 나누지 않음

**기능 폴더 안 파일 이름**
1. `handler` / `service` / `repository` / `dto`
2. `controller` / `service` / `repository` / `dto`

**백업 엔진 위치**
1. 백업 파일 만들기·이름·보관 개수 규칙은 `infra/`, 복원·두 번째 위치·백업 API는 `features/backups/`
2. `backup/`을 따로 두고 안에서 낮은 층·높은 층으로 나눔

**화면 기능 폴더**
1. 관문(로그인 전) / 업무(로그인 뒤) 둘
2. 관문 / 수강 업무 / 시스템 관리 셋

## 결정

맨 위는 2번, crate는 2번(셋), 런처 이름은 `launcher`, 서버 안은 2번, 파일 이름은 1번, 백업은 1번, 화면은 1번. 폴더 이름은 옮기는 PR에서 확정한다.

### 폴더 구조

```text
baeum-maru/
├── justfile                 # 명령은 여기 하나. cargo는 app/에서 돈다
├── app/
│   ├── Cargo.toml, Cargo.lock, rust-toolchain.toml, clippy.toml, deny.toml, .cargo/
│   ├── launcher/            # 지금 src-tauri
│   ├── server/              # 지금 src-tauri 안의 서버 쪽
│   ├── domain/              # 지금 crates/domain
│   └── ui/                  # 지금 ui
├── site/                    # 소개·다운로드·블로그 (0019)
├── docs/
└── spikes/                  # 기술 선택 실험 (ADR 근거)
```

- Rust 작업 공간과 설정 파일은 모두 `app/`에 둔다. 평소에는 저장소 맨 위에서 `just`만 쓰고, `just`가 `app/`에서 `cargo`를 부른다.
- ts-rs 출력 경로, `justfile`, CI 경로 거르기, 문서의 경로는 옮기는 PR에서 한 번에 고친다. CI는 `app/**`이 바뀌면 앱 검사, `site/**`가 바뀌면 사이트 검사로 단순해진다.
- `src-tauri`를 다른 이름으로 바꿔도 Tauri 도구가 설정 파일을 찾는지는 옮기는 PR에서 커밋 전에 확인한다: `just dev`, `just check`, Actions "Windows 빌드".
- 모듈 파일은 지금처럼 `mod.rs` 없이 `infra.rs` + `infra/` 모양으로 둔다. 아래 그림에서는 줄였다.

### crate 셋

| crate | 하는 일 | 의존 |
|---|---|---|
| `domain` (`baeum-domain`) | 순수 업무 규칙. DB·HTTP·Tauri를 모른다 (0006 그대로) | 없음 |
| `server` (`baeum-server`) | 데이터 폴더(`&Path`) 안의 모든 영속 상태를 가진다: DB, 마이그레이션, 백업·복원, 실행 모드 파일, 넘기기·넘겨받기. HTTP 서버(axum), Excel. Tauri를 모른다 | domain |
| `launcher` | 데이터 폴더 위치 정하기(설치판·포터블판), 창, 저장·고르기 대화상자, 다시 켜기, 켤 때 실행 모드로 갈라지기, 접속 PC 설정(`remote.json`). Tauri 명령은 서버의 service를 부르는 얇은 controller다 | domain, server |

- crate 사이 순환은 cargo가 막는다.
- `server`의 `lib.rs`는 launcher가 부를 것만 공개한다(서버 켜기, 실행 모드 파일, 넘기기·복원 service 등). repository와 `http` 안쪽은 crate 밖에서 보이지 않는다.
- 실행 모드 파일 함수는 DB도 HTTP 서버도 켜지 않고 `&Path`만으로 부를 수 있게 둔다. 접속 모드 PC도 쓰기 때문이다.

### launcher 안

```text
launcher/src/
├── lib.rs          # Tauri 조립, 명령 등록
├── startup.rs      # 켤 때 실행 모드로 갈라지기
├── data_dir.rs     # 설치판·포터블판 데이터 폴더
└── commands/       # Tauri 명령 = launcher의 controller
    ├── run_mode.rs, remote.rs, handover.rs, broken_db.rs, host_screen.rs
    └── files.rs    # 저장 창·고르기 창
```

### server 안: 기능별 폴더와 controller-service-repository

```text
server/
├── migrations/
└── src/
    ├── lib.rs              # 바깥에 내놓는 것만 pub
    ├── router.rs           # 기능별 routes()를 묶고 로그인 필요 여부 등을 정한다
    ├── infra/              # DB 열기·마이그레이션·검사, 시계, 암호화, 데이터 묶음, 백업 파일
    ├── http/               # ApiError, 인증, 쪽 나누기, 변경 알림(SSE), 버전 확인
    ├── features/
    │   ├── rooms/                   # 단순: handler.rs, repository.rs, dto.rs, tests.rs
    │   ├── applications/            # 복잡: + service.rs, 테스트가 크면 tests/{handler,service,repository}.rs
    │   ├── application_priorities/  # 하위 기능도 따로 기능 폴더
    │   ├── backups/                 # 복원, 백업 검사, 두 번째 위치, 백업 API
    │   ├── handover/                # 넘기기·넘겨받기·넘기기 취소, 실행 모드 파일
    │   └── ...
    ├── roster_xlsx/        # Excel 명단
    ├── seed/               # 더미 데이터 (dev-seed 기능일 때만)
    └── bin/seed.rs
```

- **handler**(controller): HTTP 요청을 받아 권한을 보고 service나 repository를 부른다. 주소 등록도 기능마다 `routes()`로 여기서 하고, `router.rs`는 묶기만 한다. 응답 모양은 `dto`.
- **service**: 트랜잭션을 열고(`pool.begin()`), 업무 규칙(`baeum-domain`)과 repository 여러 개를 엮는다. 자기 오류 enum을 돌려주고, handler와 Tauri 명령이 각자 HTTP 응답과 화면 문장으로 바꾼다.
- **repository**: SQL만. `impl SqliteExecutor<'_>`나 연결을 빌려 받는 함수다. 트랜잭션을 스스로 열지 않는다.
- service는 한 요청이 여러 테이블을 한 트랜잭션으로 다루거나 규칙 여럿을 엮을 때만 둔다: 신청, 추첨, 선발 결과, 직원·로그인, 백업·복원, 넘기기·넘겨받기. 강의실, 과목, 우선 배정 사유, 학기·강좌는 handler와 repository만 둔다.
- repository마다 trait을 만들지 않는다. 테스트는 지금처럼 임시 SQLite로 한다.
- 테스트는 기능 폴더의 `tests.rs` 하나로 두고, 커지면 `tests/` 아래 층별 파일로 나눈다.

### domain 안

개념마다 파일 하나로 평평하게 둔다(`member.rs`, `lottery.rs` …). 이름은 server의 기능 폴더와 맞춘다. 테스트는 늘 `member/tests.rs`처럼 따로 둔다.

### ui 안

```text
ui/src/
├── app/            # 진입점, 흐름 분기, 변경 알림처럼 앱 전체가 도는 것
├── api/            # 요청 도구(client, session, 앱 명령 부르기), generated/(ts-rs)
├── components/     # 공통 화면 부품 (대화창, 폼, 표, 탭, 관리자 PIN 칸)
├── lib/            # 화면 없는 순수 도구 (날짜 글자 등)
└── features/
    ├── gate/       # 로그인 전 관문: 실행 모드, 접속, 깨진 DB, 넘기기·넘겨받기, 로그인·잠금
    └── workspace/  # 로그인 뒤 화면 전부
        └── members/
            ├── api.ts        # 회원 API 부르기
            ├── components/   # MembersPage.tsx, MemberForm.tsx …
            ├── hooks/        # useMemberSearch.ts …
            └── data/         # memberFormData.ts (순수 계산) …
```

화면 테스트(`*.test.ts`)는 테스트하는 파일 옆에 둔다.

### 의존 방향 (순환 금지)

`router → features → http, infra → domain` 방향으로만 부른다. launcher는 server의 `lib.rs`가 공개한 것만 부른다.

- `infra`는 domain 말고 아무것도 부르지 않는다.
- 백업 파일 만들기, 이름, 보관 개수 규칙(지금 `backup::name`, `backup::store`, `backup::retention`)은 `infra`에 둔다. 셋 다 DB 구조를 모르고 파일만 다룬다. DB 열기의 구조 변경 직전 백업이 이것만 쓰므로 `db` ↔ `backup` 순환이 풀린다. 복원, 백업 검사, 두 번째 위치는 `features/backups`가 `infra`를 불러 한다.
- 기능 폴더끼리 부를 때는 한 방향만 허용하고 아래 표에 적는다. 표에 없는 방향은 새로 만들 때 이 표를 고친다.

| 부르는 쪽 | 부를 수 있는 쪽 |
|---|---|
| 모든 기능 | `access_logs` (다른 기능을 부르지 않는다), `staff`의 PIN 확인 |
| `applications` | `selection_results` (추가 신청 결과 계산) |
| `handover` | `staff`, `access_logs`, `backups` |

화면도 같은 원칙이다: `features/` → `components/`, `lib/`, `api/`. 기능 폴더끼리는 부르지 않고, 같이 쓰게 되면 공통 폴더로 올린다.

## 이유

- **app/과 site/**: 맨 위만 봐도 두 제품이 보인다. 넷(launcher, server, domain, ui)은 앱 하나를 만드는 데 모두 필요하고 따로 내보내지 않는다. CI 경로 거르기도 단순해진다.
- **crate 셋**: Tauri를 쓰는 곳이 창·대화상자·다시 켜기뿐이라 서버를 떼기 쉽다. 떼면 Tauri 없이 서버를 빌드하고 테스트할 수 있고, 서버가 런처를 부르는 실수가 컴파일 오류가 된다. 더 잘게 나누면 crate마다 공개 표시가 늘어나 배우는 부담이 커진다.
- **기능별 폴더**: 기능 하나를 고칠 때 한 폴더만 보면 된다(응집도). 폴더 사이 의존을 표로 관리할 수 있다. 서버와 화면이 같은 기능 이름으로 맞물린다.
- **controller-service-repository**: 유지보수자에게 익숙하고, axum에서도 흔한 모양이다. 트랜잭션 경계가 service로 모여 "어디서 트랜잭션을 여나"가 분명해진다. HTTP와 Tauri 명령이 같은 service를 불러 규칙과 문장이 한 곳에 있게 된다.
- **handler라는 이름**: axum 문서와 예제가 쓰는 말이다. controller는 HTTP와 Tauri 명령을 함께 가리키는 개념 이름으로만 쓴다.
- **백업 파일 만들기를 infra로**: 마이그레이션, 추첨, 넘기기가 모두 백업을 만들므로 맨 아래에 있어야 순환이 생기지 않는다. 나머지 백업 코드는 다른 기능처럼 한 폴더에 모인다.
- **화면 둘**: App·StaffFlow 바깥이냐 안쪽이냐로 코드 흐름이 딱 갈린다. 셋으로 나누면 넘기기(시작은 백업 탭, 진행은 관문)처럼 걸치는 기능이 생긴다.

## 결과

- 리팩터링은 동작을 바꾸지 않는다. 요청 주소, SSE 알림, DB 구조, ts-rs 타입은 그대로다.
- PR 순서
  1. 지금 crate 안에서 경계 맞추기: `db` ↔ `backup` 순환 풀기, 넘기기·넘겨받기·DB 검사 본체를 서버 쪽 service 모양으로 빼기, 실행 모드 파일과 데이터 폴더 경계 정리
  2. `server` crate 나누기와 `app/` 아래로 폴더 옮기기 (옮기기만)
  3. 서버·domain을 기능별 폴더로 재편, 테스트를 `tests.rs`로 분리
  4. 공개 API로 도는 흐름 테스트(`app/server/tests/`). 서버 테스트 도우미는 `test-support` cargo 기능으로 열고 출시 빌드에서는 끈다
  5. 화면 폴더 정리
- `docs/conventions.md`의 "모듈 파일 배치"와 테스트 위치, `justfile`, CI 경로, `docs/development.md`, `AGENTS.md`의 경로를 각 PR에서 함께 고친다.
- 0006의 "앱 crate 하나에 나머지 전부"는 이 기록으로 대체한다. 업무 규칙 crate의 원칙은 그대로다.
