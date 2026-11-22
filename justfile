# 배움마루 개발 명령. 목록 보기: just

generated := "ui/src/api/generated"
seed_dir := "target/debug/data"

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

# 화면 확인용 더미 데이터를 넣는다. 개발 exe 옆 data 폴더라서 just dev가 포터블판으로 이 폴더를 쓴다.
# 폴더가 비어 있지 않으면 아무것도 넣지 않고 멈춘다
seed:
    cargo run -p baeum-maru --features dev-seed --bin seed -- {{seed_dir}}

# 더미 데이터 폴더를 지운다. just dev가 다시 원래 데이터 폴더를 쓴다
seed-clear:
    rm -rf {{seed_dir}}

# 전부 검사한다. CI도 이 명령을 쓴다
check: check-names check-rust check-bindings check-ui check-deps

# Windows는 대소문자를 가리지 않고, import는 확장자 없이 찾는다.
# MemberForm.tsx와 memberForm.ts가 같은 폴더에 있으면 Windows 빌드만 깨진다.
check-names:
    #!/usr/bin/env bash
    set -euo pipefail
    same=$(git ls-files --cached --others --exclude-standard \
        | sed -E 's/\.(tsx?|jsx?)$//' | tr '[:upper:]' '[:lower:]' | sort | uniq -d)
    if [ -n "$same" ]; then
        echo "Windows에서 같은 이름으로 보이는 파일이 있습니다:"
        echo "$same"
        exit 1
    fi

check-rust:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features

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
    npm --prefix ui run --silent test

check-deps:
    cargo deny check
