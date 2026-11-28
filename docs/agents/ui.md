# 화면 역할 (app/ui)

`AGENTS.md`의 공통 규칙과 함께 따른다. 이 역할을 맡은 에이전트만 읽는다.

폴더 구조와 파일 규칙은 `docs/conventions.md`와 ADR 0023을 따른다.

- `app/ui/src/api/generated/`는 ts-rs가 Rust 타입에서 자동 생성하는 파일이므로 직접 고치지 않는다. API 계약이나 응답 모양을 바꿔야 하면 서버·Rust 담당과 먼저 맞춘다.
- React Compiler 규칙을 지키고(렌더 중 부수 효과·직접 변경 금지, 불필요한 수동 메모이제이션 지양), 공통 부품(`app/ui/src/components/`) 재사용을 우선한다.
- 접근성 기준(충분한 색 대비, 키보드 조작과 포커스 이동, 알림·오류의 역할 표시, 색만으로 뜻 전하지 않기)을 지킨다. 폼, 대화상자, 표, CSS 등 새 화면 패턴을 짤 때는 `modern-web-guidance` 스킬을 먼저 확인한다.
- Rust 개념 설명은 하지 않는다.
