# 배움마루 개발 명령. 목록 보기: just

generated := "ui/src/api/generated"

default:
    @just --list

# 처음 한 번, 그리고 package-lock.json이 바뀌었을 때
setup:
    npm ci --prefix ui

# 개발 모드: Vite와 앱 창을 함께 띄운다
dev:
    ui/node_modules/.bin/tauri dev

# 실제 앱 빌드 (설치 파일 없이)
build:
    ui/node_modules/.bin/tauri build --no-bundle

# 전부 검사한다. CI도 이 명령을 쓴다
check: check-rust check-bindings check-ui check-deps

check-rust:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# [ADR 0012]: ts-rs 생성 파일이 Rust와 다르면 실패
check-bindings:
    rm -rf {{generated}}
    cargo test --workspace --quiet export_bindings
    git diff --exit-code -- {{generated}}
    test -z "$(git ls-files --others --exclude-standard -- {{generated}})"

check-ui:
    npm --prefix ui run --silent format:check
    npm --prefix ui run --silent lint
    npm --prefix ui run --silent typecheck

check-deps:
    cargo deny check
