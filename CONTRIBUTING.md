# Contributing

배움마루는 개인 사이드 프로젝트로 개발 중입니다. 기여를 받더라도 작은 단위의 수정과 재현 가능한 설명을 우선합니다.

## 개발 준비

```powershell
cd backend
uv sync --all-groups
uv run ruff format --check .
uv run ruff check .
uv run mypy
uv run pytest

cd ../frontend
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

직원용 앱은 `pnpm dev:operator`, 런처 앱은 `pnpm dev:launcher`로 실행합니다.

## PR 기준

- 한 PR은 하나의 목적에 집중합니다.
- 기능 변경은 가능한 한 테스트를 함께 추가합니다.
- DB 변경은 migration과 함께 설명합니다.
- 실사용 개인정보, DB 파일, 백업 파일, 엑셀 출력 파일을 포함하지 않습니다.
- UI 변경은 주요 화면을 직접 열어 확인합니다.

## 커밋 메시지

예시는 다음 형식을 사용합니다.

```text
feat: 신청 상태 관리 추가
- 확정 처리 지원
- 취소 후 대기자 승격 처리
```

## 개인정보

이 프로젝트는 업무상 개인정보를 다룰 수 있습니다. 이슈, PR, 스크린샷, 테스트 데이터에는 실제 개인정보를 넣지 않습니다.
