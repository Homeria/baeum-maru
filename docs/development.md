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
```

`just`만 치면 전체 목록이 나온다.

`just check`가 하는 일:

1. Rust: 형식(rustfmt), clippy(경고도 실패), 테스트
2. ts-rs 생성 파일(`ui/src/api/generated/`)을 지우고 다시 만들어 레포에 올린 것과 비교
3. 화면: 형식(Prettier), ESLint, 타입 검사
4. 의존성: cargo-deny (보안 취약점, 라이선스, 출처)

## 자주 걸리는 것

- **ts-rs 비교 실패**: 응답 구조체를 고치고 생성 파일을 커밋하지 않았다. `just check-bindings`가 다시 만든 파일을 확인하고 함께 스테이징한다.
- **cargo-deny 라이선스 실패**: 새 의존성의 라이선스가 `deny.toml`의 허용 목록에 없다. 넣어도 되는 라이선스인지 먼저 확인한다.
