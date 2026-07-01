# 배움마루

배움마루는 복지관, 문화센터, 평생교육기관의 수강신청, 추첨, 대기자, 출석부, 엑셀 출력 업무를 내부망에서 처리하기 위한 포터블 로컬 호스팅 업무 도구입니다.

현재 저장소는 로컬 실행형 MVP의 핵심 업무 흐름을 기능 단위로 구현 중입니다.

## 개발 방향

- Windows 사무용 노트북에서 실행 가능한 포터블 앱을 우선합니다.
- Go 내장 HTTP 서버와 SQLite를 중심으로 구현합니다.
- 현재 포터블 실행 파일은 콘솔형 런처가 서버를 시작하고 브라우저를 여는 방식입니다.
- 네이티브 GUI 런처는 MVP 이후 사용성 개선 단계에서 검토합니다.
- 포터블 패키징은 Windows ZIP 배포를 우선합니다.
- 자세한 설계는 [docs/00_README.md](docs/00_README.md)를 기준으로 합니다.

## 현재 구현 상태

- 설정 로더, 파일 로깅, SQLite 연결, 마이그레이션
- 회원/강좌 관리와 수강신청 등록/취소
- 신청 제한 규칙과 신청 상태 이력
- 추첨, 대기자, 결과 조회, 결과 엑셀 내보내기
- 출석 세션/출석 기록과 출석 엑셀 내보내기
- 수동 백업, 복구 예약, 시작 시 복구 적용
- Windows 포터블 패키징 스크립트
- GitHub Actions 기반 Go 빌드/테스트 CI

## 개발 실행

개발 중에는 다음 명령으로 로컬 서버를 실행할 수 있습니다.

```powershell
go run ./cmd/server
```

## Windows 포터블 패키징

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package_windows.ps1 -Version 0.1.0
```

생성된 `dist/BaeumMaru_Portable_<version>/baeum-maru.exe`를 실행하면 로컬 서버가 시작되고 브라우저가 열립니다.

GitHub Actions의 `Windows Package` workflow를 수동 실행하거나 `v*` 태그를 push하면 같은 ZIP 패키지를 artifact로 받을 수 있습니다.

