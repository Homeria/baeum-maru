# 작업 단위 가이드

이 문서는 Codex가 각 브랜치를 구현할 때 지켜야 할 공통 규칙을 기록한다. 전체 순서는 `18_prototype_branch_roadmap.md`가 단일 기준이다.

## 브랜치 시작 전

- 현재 branch와 dirty worktree를 확인한다.
- `13_current_state.md`와 `18_prototype_branch_roadmap.md`에서 현재 위치와 선행 조건을 확인한다.
- 최신 `develop`에서 로드맵에 적힌 이름으로 branch를 만든다.
- 이전 단계가 merge되지 않았다면 다음 단계로 넘어가지 않는다.
- `main`은 사용자가 명시적으로 요청하지 않는 한 checkout, PR, merge 대상에 포함하지 않는다.

## Python 전환 규칙

- Go 구현은 `go-prototype-baseline-2026-07` 태그에서만 참고한다.
- Go 파일을 Python 문법으로 기계 번역하지 않고 스키마와 업무 규칙에서 새 구현을 작성한다.
- Go/Fyne/template compatibility layer를 활성 구현에 남기지 않는다.
- 실사용 데이터가 없으므로 과거 Go DB에서 변환하는 migration을 작성하지 않는다.
- Alembic은 최신 기준 스키마 하나에서 시작한다.

## 구현 원칙

- branch 하나는 하나의 사용자 가치 또는 하나의 architecture boundary만 바꾼다.
- FastAPI router, Pydantic schema, application service, repository 책임을 분리한다.
- Pydantic validation만 믿지 않고 업무 rule과 DB constraint를 test한다.
- 여러 repository를 바꾸는 use case는 unit of work transaction 안에서 수행한다.
- API 변경은 success, validation, permission, conflict response를 test한다.
- UI 변경은 자동 test 후 Windows desktop과 좁은 화면에서 확인한다.
- package 관련 변경은 Python 미설치 Windows의 실제 artifact로 확인한다.
- Python/Node dependency를 바꾸면 lockfile과 license 영향을 함께 갱신한다.

## 기본 검증

Python 기반이 생성된 이후 branch 종료 전 다음 검사를 기본으로 수행한다.

```text
uv lock --check
uv run ruff format --check .
uv run ruff check .
uv run mypy backend
uv run pytest
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

변경 범위에 따라 Playwright, Alembic 빈 DB upgrade, PyInstaller Windows build, multi-client test를 추가한다.

## 브랜치 종료 기준

- 관련 test와 전체 기본 검사가 통과한다.
- 현재 상태와 실제 구현이 달라졌다면 같은 branch에서 문서를 갱신한다.
- GitHub Action이 필요하면 같은 branch에서 추가하거나 수정한다.
- 수동 확인이 필요한 branch는 사용자의 확인 전 PR merge를 멈춘다.
- PR과 CI가 통과한 뒤 `develop`에 merge하고 작업 branch를 삭제한다.

## 사용자 수동 검증

로드맵에서 `수동 확인`으로 표시한 단계는 Codex가 실행 artifact 또는 test 환경을 준비하고, 사용자가 실제 조작한 피드백을 받은 뒤 완료로 판단한다.
