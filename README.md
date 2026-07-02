# 배움마루

배움마루는 복지관, 문화센터, 평생교육기관의 수강신청, 추첨, 대기자, 출석부, 엑셀 출력 업무를 내부망에서 처리하기 위한 포터블 로컬 호스팅 업무 도구입니다.

현재 저장소는 개인 사이드 프로젝트로 진행 중인 로컬 실행형 MVP입니다.

## 기능

- 회원 등록과 검색
- 강좌 개설 관리
- 수강신청 등록, 취소, 확정
- 중복 신청, 시간대 충돌, 최대 신청 수 제한
- 강좌별 추첨과 대기자 승격
- 출석 회차 생성과 출석 기록
- 회원, 강좌, 신청, 추첨 결과, 출석 엑셀 출력
- 엑셀 회원/강좌 가져오기
- SQLite 백업, 복구 예약, 백업 보관 상태 표시
- 운영 설정 화면과 감사 로그
- Windows 포터블 ZIP 패키징

## 기술 스택

- Go
- SQLite
- Go 내장 HTTP 서버
- HTML template 기반 관리자 화면
- GitHub Actions CI

자세한 설계는 [docs/00_README.md](docs/00_README.md)를 기준으로 합니다.

## 개발 실행

```powershell
go test ./...
go run ./cmd/server
```

기본 실행 후 브라우저에서 다음 주소로 접속합니다.

```text
http://127.0.0.1:18080
```

## Windows 포터블 패키징

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package_windows.ps1 -Version 0.1.0
```

생성된 `dist/BaeumMaru_Portable_<version>/baeum-maru.exe`를 실행하면 로컬 서버가 시작되고 브라우저가 열립니다.

GitHub Actions의 `Windows Package` workflow를 수동 실행하거나 `v*` 태그를 push하면 같은 ZIP 패키지를 artifact로 받을 수 있습니다.

## 데이터 주의

이 앱은 회원명, 연락처, 생년월일, 신청 내역, 출석 기록 같은 개인정보를 다룰 수 있습니다.

- `data/`, `backups/`, `exports/`, `imports/`, `logs/` 폴더는 Git에 올리지 않습니다.
- `.db`, `.xlsx`, `.csv` 같은 업무 데이터 파일은 저장소에 포함하지 않습니다.
- 공개 이슈나 PR에는 실사용자 개인정보를 올리지 않습니다.
- 스크린샷을 공유할 때는 개인정보를 제거합니다.

## 저장소 구조

```text
cmd/          실행 진입점
internal/     애플리케이션 코드
docs/         기획, 설계, 업무 규칙 문서
scripts/      로컬 패키징 스크립트
web/          정적 파일과 템플릿 자리
testdata/     테스트 전용 데이터 자리
```

## 라이선스

이 프로젝트는 [PolyForm Noncommercial License 1.0.0](LICENSE)를 따릅니다.

비상업적 목적의 사용, 복사, 수정, 포크, 배포를 허용합니다. 배움마루를 기반으로 한 파생 프로젝트와 재배포본은 [NOTICE](NOTICE)와 `LICENSE`의 `Required Notice`를 유지하여 사용자와 개발자가 배움마루가 원 프로젝트임을 확인할 수 있게 해야 합니다.

상업적 사용 권한은 이 라이선스에 포함되지 않습니다.
