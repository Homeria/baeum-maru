# 배움마루

배움마루는 복지관, 문화센터, 평생교육기관의 수강신청, 추첨, 대기자, 출석부, 엑셀 출력 업무를 내부망에서 처리하기 위한 포터블 로컬 호스팅 업무 도구입니다.

현재 저장소는 로컬 실행형 MVP를 기능 단위로 구현 중입니다.

## 개발 방향

- Windows 사무용 노트북에서 실행 가능한 포터블 앱을 우선합니다.
- 초기 구현은 Go 내장 HTTP 서버와 SQLite를 중심으로 진행합니다.
- 포터블 패키징은 Windows ZIP 배포를 우선합니다.
- 자세한 설계는 [docs/00_README.md](docs/00_README.md)를 기준으로 합니다.

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

