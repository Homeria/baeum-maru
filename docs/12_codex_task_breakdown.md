# 작업 단위 가이드

이 문서는 Codex가 각 브랜치를 구현할 때 지켜야 할 공통 규칙을 기록한다. 전체 브랜치 순서는 `18_prototype_branch_roadmap.md`가 단일 기준이다.

## 브랜치 시작 전

- 현재 브랜치와 dirty worktree를 확인한다.
- `13_current_state.md`와 `18_prototype_branch_roadmap.md`에서 현재 위치와 선행 조건을 확인한다.
- 최신 `develop`에서 로드맵에 적힌 이름으로 브랜치를 만든다.
- 이전 단계가 merge되지 않았다면 다음 단계로 넘어가지 않는다.

## 구현 원칙

- 브랜치 하나는 하나의 사용자 가치 또는 하나의 아키텍처 경계만 바꾼다.
- 구조 리팩터링은 characterization test를 유지하고 동작을 바꾸지 않는다.
- DB 변경은 스키마 문서, 서비스 테스트, API/UI 영향을 함께 점검한다.
- Huma, React, Wails 타입을 도메인과 application 계층으로 유출하지 않는다.
- API 변경은 성공, 검증 오류, 권한 오류, 동시성 충돌을 테스트한다.
- UI 변경은 자동 테스트 후 Windows 데스크톱과 좁은 화면에서 수동 확인한다.
- 새 구현이 검증되기 전에는 기존 Fyne/template fallback을 제거하지 않는다.

## 브랜치 종료 기준

- 관련 테스트와 전체 `go test ./...`가 통과한다.
- 필요에 따라 `go test -race ./...`, frontend test/build, Wails Windows build를 실행한다.
- 문서의 현재 상태와 실제 구현이 달라졌다면 같은 브랜치에서 갱신한다.
- PR과 CI가 통과한 뒤 `develop`에 merge하고 다음 브랜치로 이동한다.

## 사용자 수동 검증

로드맵에서 `수동 확인`으로 표시한 단계는 Codex가 실행 파일 또는 테스트 환경을 준비하고, 사용자가 실제 조작한 피드백을 받은 뒤 완료로 판단한다.
