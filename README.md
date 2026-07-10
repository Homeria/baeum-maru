# 배움마루

배움마루는 복지관, 문화센터, 평생교육기관이 내부망에서 회원 관리, 수강신청, 추첨, 출석, Excel 출력을 처리하도록 돕는 로컬 호스팅 업무 시스템입니다.

현재는 Go와 SQLite 기반의 기능 검증 구현이 동작하며, 프로토타입은 `Go + net/http + Huma v2` API, React/Vite 웹 화면, Wails Windows 런처로 전환합니다. 직원은 별도 앱 설치 없이 브라우저로 접속하고, 호스트 담당자만 전용 런처를 사용합니다.

## 현재 기능

- 회원, 강좌 개설, 수강신청, 신청 제한 규칙
- 추첨, 대기자, 출석, Excel 가져오기/내보내기
- SQLite 백업/복구, 감사 로그, 접속 코드와 역할 권한
- Fyne 기반 운영 런처와 Windows portable ZIP 기반

## 목표 프로토타입

- React/Vite/TypeScript 기반 접수 및 운영 웹
- Huma v2와 OpenAPI 3.1 기반 REST API 및 생성된 TypeScript API 타입
- 회원 정보와 다중 강좌 선택을 한 번에 저장하는 접수 흐름
- SSE 기반 다중 사용자 갱신
- Wails v2 기반 Windows 호스트 런처
- HTTPS, CSRF, 로그인 실패 제한, Windows 실기기 운영 검증
- WebView2 런타임 부재 시 설치 안내와 콘솔 서버 fallback을 포함한 포터블 패키지

## 개발 검증

```powershell
go test ./...
go test -race ./...
go vet ./...
go build ./cmd/server
```

자세한 설계와 작업 순서는 [docs/00_README.md](docs/00_README.md)를 참고합니다.

## 데이터 주의

회원명, 연락처, 생년월일, 신청 내역, 출석 기록, 백업 파일은 개인정보를 포함할 수 있습니다. `data/`, `backups/`, `exports/`, `imports/`, `logs/`와 업무용 파일은 저장소에 올리지 않습니다.

## 라이선스

이 프로젝트는 [PolyForm Noncommercial License 1.0.0](LICENSE)를 따릅니다. 비상업적 사용과 개선 포크를 허용하며, 파생 프로젝트는 `LICENSE`, `NOTICE`, 원 프로젝트 출처를 유지해야 합니다.
