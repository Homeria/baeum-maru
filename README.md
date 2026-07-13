# 배움마루

배움마루는 복지관, 문화센터, 평생교육기관이 내부망에서 회원 관리, 수강신청, 추첨, 출석, Excel 출력을 처리하도록 돕는 로컬 호스팅 업무 시스템입니다.

## 현재 전환 상태

기존 Go/Fyne 기능 검증 구현은 `go-prototype-baseline-2026-07` 태그에 보존했습니다. 실사용·배포 데이터가 없는 현재 시점부터 활성 구현을 `Python + FastAPI + SQLite + React/Vite + pywebview` 기반으로 전면 교체합니다.

- 기존 Go 코드는 참고 구현이며 새 기능을 추가하지 않습니다.
- Python 전환 작업은 `develop`에만 누적합니다.
- 사용자 요청 전에는 `main`을 변경하지 않습니다.
- 최신 DB 스키마, 업무 규칙, UI 피드백은 Python 구현의 요구사항으로 이어갑니다.

## 목표 프로토타입

- FastAPI/Pydantic 기반 `/api/v1` REST API와 OpenAPI 문서
- SQLAlchemy 2, Alembic, SQLite WAL 기반 로컬 데이터 저장
- React/Vite/TypeScript 기반 접수 및 운영 웹
- 회원 정보와 여러 희망 강좌를 한 번에 저장하는 접수 흐름
- FastAPI WebSocket과 TanStack Query를 이용한 다중 사용자 갱신
- pywebview 독립 런처와 내부망 업무 서버 프로세스 분리
- 접속 코드, 역할 권한, HTTPS, CSRF, 로그인 실패 제한
- PyInstaller `onedir` 기반 Windows 포터블 ZIP
- Docker 기반 중앙 서버 배포로 확장 가능한 구조

## 보존된 Go 기준점

```text
tag: go-prototype-baseline-2026-07
commit: 547977b13d77ffc0dbaa42a4dd4c24829a000d6f
```

태그에는 Go, Go template, Fyne로 구현한 회원·강좌·신청·추첨·출석·Excel·백업 기능이 남아 있습니다. 활성 작업 트리에서 Go 코드가 제거된 뒤에도 설계 의도와 과거 동작을 확인할 수 있습니다.

자세한 기술 결정과 브랜치 순서는 [docs/00_README.md](docs/00_README.md)를 참고합니다.

## 데이터 주의

회원명, 연락처, 생년월일, 신청 내역, 출석 기록, 백업 파일은 개인정보를 포함할 수 있습니다. `data/`, `backups/`, `exports/`, `imports/`, `logs/`와 업무용 파일은 저장소에 올리지 않습니다.

## 라이선스

이 프로젝트는 [PolyForm Noncommercial License 1.0.0](LICENSE)를 따릅니다. 비상업적 사용과 개선 포크를 허용하며, 파생 프로젝트는 `LICENSE`, `NOTICE`, 원 프로젝트 출처를 유지해야 합니다.
