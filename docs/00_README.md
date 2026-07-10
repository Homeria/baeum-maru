# 배움마루 문서 모음

배움마루는 복지관, 문화센터, 평생교육기관의 수강신청 업무를 내부망에서 처리하는 로컬 호스팅 시스템이다. 호스트 노트북은 전용 런처를 실행하고, 접수 직원은 같은 네트워크에서 브라우저로 접속한다.

## 현재와 목표

- 현재 구현: Go, SQLite, Go HTML template, Fyne 런처를 사용한 기능 검증 기반
- 목표 프로토타입: Go, `net/http`, Huma v2, SQLite, React/Vite/TypeScript, Wails v2, SSE
- 사용자 화면은 설치 없이 브라우저로 사용하며, Wails는 호스트 노트북의 운영 콘솔에만 사용한다.

## 문서 읽는 순서

1. `01_project_summary.md`: 프로젝트 목적과 운영 방식
2. `02_requirements.md`: 사용자, 업무, 보안, 비기능 요구사항
3. `03_tech_stack.md`: 확정 기술 스택과 선택 이유
4. `04_runtime_architecture.md`: 런처, 서버, 웹 클라이언트의 관계
5. `05_file_structure.md`: 현재 구조와 목표 구조
6. `06_development_plan.md`: 프로토타입 완성 순서
7. `07_mvp_scope.md`: 기능 MVP와 프로토타입 완료 기준
8. `08_data_model.md`, `16_schema_redesign_notes.md`: 데이터 모델과 개편 원칙
9. `09_screen_flows.md`, `10_business_rules.md`: 화면과 업무 규칙
10. `11_quality_checklist.md`, `14_release_checklist.md`: 검증과 배포 점검
11. `13_current_state.md`: 실제 구현 상태와 다음 작업
12. `17_prototype_architecture.md`: React/Wails 전환 아키텍처 결정

## 문서 운영 원칙

- `13_current_state.md`는 코드와 테스트 결과에 맞춰 갱신한다.
- 현재 구현과 목표 구조를 섞어 쓰지 않는다. 계획 항목에는 `목표`, 구현된 항목에는 `현재`를 붙인다.
- 스키마 대개편은 실사용 데이터가 없다는 전제에서만 기존 DB를 폐기하고 진행한다.
- 기능을 빨리 끝내는 것보다, 실제 기관 업무를 검증할 수 있는 구조와 경험을 우선한다.
- 기술 스택은 호환성, 명시성, 포크 가능성을 우선하며 `03_tech_stack.md`의 채택 결정을 따른다.
