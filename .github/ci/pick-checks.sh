#!/usr/bin/env bash
# [ADR 0024]: 바뀐 파일 목록을 받아 돌릴 앱 검사 작업을 고른다.
# 쓰는 법: git diff --name-only 기준 HEAD | pick-checks.sh   (한 줄에 파일 하나)
#          pick-checks.sh all                                 (셋 다)
# 출력: check.yml 매트릭스의 include에 넣을 JSON 배열 한 줄
set -euo pipefail

ui=false
rust=false
deps=false

all() { ui=true; rust=true; deps=true; }

if [ "${1:-}" = all ]; then
  all
else
  while IFS= read -r file; do
    case "$file" in
      "") ;;
      # 문서는 경로 조건에서도 빠진다. 다른 파일과 함께 바뀌어도 검사를 늘리지 않는다
      *.md) ;;
      # 자동 생성 파일은 Rust 타입과 맞는지(check-bindings)도 본다
      app/ui/src/api/generated/*) ui=true; rust=true ;;
      app/ui/* | .node-version) ui=true ;;
      # 화면도 버전 번호를 workspace Cargo.toml에서 읽는다
      app/Cargo.toml) all ;;
      app/Cargo.lock | app/*/Cargo.toml) rust=true; deps=true ;;
      app/deny.toml) deps=true ;;
      app/domain/* | app/server/* | app/launcher/* | app/.cargo/* \
        | app/rust-toolchain.toml | app/clippy.toml) rust=true ;;
      justfile | .github/workflows/check.yml | .github/ci/*) all ;;
      # 위에서 나누지 않은 app/ 파일은 어디에 닿는지 모르므로 셋 다 돌린다
      app/*) all ;;
      # 그 밖(다른 워크플로, site/, docs/ 등)은 앱 검사와 상관없다
      *) ;;
    esac
  done
fi

entries=()
if $ui; then
  entries+=('{"kind":"ui","name":"앱 화면 (React)","tools":"just","recipes":"check-ui"}')
fi
if $deps; then
  entries+=('{"kind":"deps","name":"Rust 의존성 (보안·라이선스)","tools":"just,cargo-deny","recipes":"check-deps"}')
fi
if $rust; then
  entries+=('{"kind":"rust","name":"앱 본체 (Rust: 도메인·서버·런처)","tools":"just","recipes":"check-rust check-bindings"}')
fi

(IFS=,; echo "[${entries[*]}]")
