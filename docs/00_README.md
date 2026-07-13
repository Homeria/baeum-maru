# 배움마루 문서 모음

배움마루는 복지관, 문화센터, 평생교육기관의 수강신청 업무를 내부망에서 처리하는 로컬 호스팅 시스템이다. 한 대의 Windows 노트북이 호스트가 되고, 여러 직원은 같은 네트워크에서 브라우저로 접속한다.

## 현재와 목표

- 보존 구현: `go-prototype-baseline-2026-07` 태그의 Go/SQLite/Go template/Fyne 프로토타입
- 활성 전환: Go 구현을 제거하고 Python/FastAPI 기반을 새로 구성하는 단계
- 목표 프로토타입: Python 3.13, FastAPI, Pydantic v2, SQLAlchemy 2, SQLite, React/Vite/TypeScript, pywebview, WebSocket
- 호스트 제어: pywebview와 WebView2 기반 독립 런처를 사용하며 직원 화면은 내부망 브라우저로 제공한다.
- 배포: Python 설치가 필요 없는 PyInstaller `onedir` 포터블 ZIP을 기본으로 한다.

## 문서 읽는 순서

1. `01_project_summary.md`: 프로젝트 목적과 운영 방식
2. `02_requirements.md`: 사용자, 업무, 보안, 비기능 요구사항
3. `03_tech_stack.md`: 확정 Python 기술 스택과 선택 이유
4. `17_prototype_architecture.md`: Python 전환 결정과 아키텍처 경계
5. `04_runtime_architecture.md`: 호스트 제어면, 업무 서버, 브라우저의 관계
6. `05_file_structure.md`: 목표 파일 구조와 의존 방향
7. `06_development_plan.md`: 프로토타입 완성 단계
8. `18_prototype_branch_roadmap.md`: 전체 브랜치 순서와 수동 검증 지점
9. `08_data_model.md`, `16_schema_redesign_notes.md`: 데이터 모델과 개편 원칙
10. `09_screen_flows.md`, `10_business_rules.md`: 화면과 업무 규칙
11. `11_quality_checklist.md`, `14_release_checklist.md`: 검증과 배포 점검
12. `13_current_state.md`: 실제 구현 상태와 바로 다음 작업

## 문서 운영 원칙

- `13_current_state.md`는 코드와 테스트 결과에 맞춰 갱신한다.
- 다음 브랜치를 만들기 전에 `18_prototype_branch_roadmap.md`를 확인한다.
- 현재 구현과 목표 구조를 섞어 쓰지 않는다.
- Go 기준점은 태그에서만 보존하고 활성 구현에는 이중 유지보수 경로를 만들지 않는다.
- 아직 실사용 데이터가 없으므로 초기 Alembic migration은 최신 스키마 하나를 기준으로 작성한다.
- 기능 완성 속도보다 사용자가 코드를 이해하고 업무 규칙을 검증할 수 있는 구조를 우선한다.
- 기술 스택 변경은 `17_prototype_architecture.md`에 이유와 영향을 기록한다.
