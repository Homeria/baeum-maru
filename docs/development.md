# 개발 환경

배움마루를 고치고 빌드하는 사람을 위한 안내다. 쓰는 사람용 안내는 [README](../README.md)에 있다.

## 설치할 것

| 도구 | 버전 | 설치 |
|---|---|---|
| Rust | `rust-toolchain.toml`이 정함 | [rustup](https://rustup.rs/). 레포에서 처음 `cargo`를 실행하면 맞는 버전이 깔린다 |
| Node.js | 22 (`.node-version`) | [nodejs.org](https://nodejs.org/) 또는 버전 관리 도구 |
| just | 최신 | `cargo install --locked just` |
| cargo-deny | 최신 | `cargo install --locked cargo-deny` |

Linux에서는 Tauri가 쓰는 시스템 라이브러리도 필요하다. Ubuntu/Debian:

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

다른 운영체제는 [Tauri 문서](https://v2.tauri.app/start/prerequisites/)를 따른다.

## 명령

```sh
just setup    # 처음 한 번: 화면 패키지 설치
just dev      # 개발 모드: Vite와 앱 창을 함께 띄움
just check    # 전부 검사. PR 전에 돌린다
just build    # 실제 앱 빌드 (설치 파일 없이)
just seed     # 화면 확인용 더미 데이터 넣기 (아래 참고)
just seed-clear  # 더미 데이터 지우기
```

`just`만 치면 전체 목록이 나온다.

`just check`가 하는 일:

1. 파일 이름: 대소문자와 확장자(`.ts`, `.tsx`)만 다른 파일이 같은 폴더에 있으면 실패 (Windows에서 같은 파일로 보임)
2. Rust: 형식(rustfmt), clippy(경고도 실패), 테스트
3. ts-rs 생성 파일(`ui/src/api/generated/`)을 지우고 다시 만들어 레포에 올린 것과 비교
4. 화면: 형식(Prettier), ESLint, 타입 검사, 테스트(Vitest)
5. 의존성: cargo-deny (보안 취약점, 라이선스, 출처)

## 더미 데이터

빈 DB에서 화면을 확인하려면 `just seed`로 더미 데이터를 넣는다. 모두 지어낸 데이터다.

- `target/debug/data/`에 새 DB를 만든다. 개발 모드 실행 파일 옆 `data` 폴더라서 `just dev`가 포터블판으로 보고 이 폴더를 쓴다.
- 원래 데이터 폴더로 돌아가려면 `just seed-clear`로 지운다.
- 폴더가 비어 있지 않으면 아무것도 넣지 않고 멈춘다. 다시 넣으려면 먼저 지운다.
- 개발용 `dev-seed` 기능을 켜야 빌드되므로 출시 빌드에는 들어가지 않는다.

넣는 것:

| 종류 | 내용 |
|---|---|
| 직원 | 관리자(PIN `123456`), 직원, 임시직원, 조회전용(PIN `1234`). 관리자 복구 코드는 `just seed`가 끝날 때 보여 준다 |
| 학기 | 오늘이 들어가는 학기 하나 |
| 강의실·과목 | 본관·별관 강의실 9개(지하, 층 없음 포함), 과목 9개(분류 없는 과목 포함) |
| 강좌 | 12개. 정원 5~30명, 주 2회 강좌, 같은 과목의 여러 수준·반 |
| 회원 | 300명. 동명이인 3쌍, 생년월일·성별을 비운 회원 포함 |
| 신청 | 강좌마다 신청 없음, 정원 미만, 같음, 조금 넘음, 두 배 가까이. 규칙 예외 3건, 삭제 기록 5건 |

추첨은 하지 않은 상태다. 데이터를 바꾸려면 `src-tauri/src/seed/`를 고친다.

## 자주 걸리는 것

- **ts-rs 비교 실패**: 응답 구조체를 고치고 생성 파일을 커밋하지 않았다. `just check-bindings`가 다시 만든 파일을 확인하고 함께 스테이징한다.
- **cargo-deny 라이선스 실패**: 새 의존성의 라이선스가 `deny.toml`의 허용 목록에 없다. 넣어도 되는 라이선스인지 먼저 확인한다.
