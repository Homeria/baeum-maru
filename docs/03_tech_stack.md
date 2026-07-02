# 배움마루 기술스택 정의서

## 1. 기술 선택 기준

배움마루의 기술스택은 다음 조건을 기준으로 선택한다.

- 일반 사무용 Windows 노트북에서 실행 가능
- 설치 없이 포터블 실행 가능
- `exe` 더블클릭으로 포터블 런처 실행
- MVP에서는 런처가 내부 웹서버를 자동 시작
- 네이티브 컨트롤 패널은 후속 사용성 개선 후보
- 같은 내부망의 최대 5대 기기가 브라우저로 접속
- DB 서버 설치 없이 로컬 파일 DB 사용
- 엑셀 입출력 지원
- 추후 Linux 서버 또는 Docker 배포로 확장 가능
- 비상업 소스 공개 프로젝트로 공개 가능

## 2. 최종 추천 스택

```text
Language: Go
Portable Launcher: Go console entrypoint
Desktop Launcher GUI: Fyne, `fyne` build tag
HTTP Server: net/http + chi
Database: SQLite
DB Access: database/sql
SQLite Driver: modernc.org/sqlite
Web UI: Go html/template + HTMX
CSS: Pico CSS 또는 Bootstrap
Excel: Excelize
Config: JSON
Logging: slog 또는 zerolog
Packaging: Portable ZIP
Future Server Deployment: Linux binary / Docker
```

## 3. Go를 사용하는 이유

## 3.1 포터블 배포에 유리

Go는 컴파일 언어이고 실행 파일 배포가 비교적 단순하다.

배움마루는 담당 직원이 직접 실행해야 하므로 다음 설치를 요구하지 않아야 한다.

- Python
- Node.js
- Docker Desktop
- PostgreSQL
- 별도 런타임

Go를 사용하면 단일 실행 파일 중심의 포터블 배포가 가능하다.

## 3.2 내장 웹서버 구현이 자연스러움

Go의 `net/http`는 내장 웹서버 구현에 적합하다.  
추가 라우팅은 `chi`를 사용한다.

주요 역할:

- 내부망 HTTP 서버
- 관리자 화면 제공
- 접수 화면 제공
- 신청/추첨 API
- 엑셀 다운로드
- heartbeat 수신
- 정적 파일 제공

## 3.3 추후 서버형 전환이 쉬움

Go는 Windows exe뿐만 아니라 Linux binary도 쉽게 만들 수 있다.  
MVP는 포터블 Windows 앱으로 만들고, 추후 상시 서버형으로 확장할 수 있다.

예상 확장:

```text
Portable Mode:
Windows + console launcher + SQLite

Server Mode:
Linux binary or Docker + SQLite/PostgreSQL
```

## 4. 런처 선택

MVP에서는 Go 콘솔형 런처를 우선 사용한다.

이유:

- 포터블 실행 파일 크기를 작게 유지할 수 있다.
- GUI 의존성을 늦게 결정할 수 있다.
- 서버, DB, 백업, 업무 흐름을 먼저 검증할 수 있다.
- Windows ZIP 배포를 빠르게 확인할 수 있다.

Fyne은 Go 기반 GUI 툴킷이며, 네이티브 런처 컨트롤 패널을 만들 때 사용한다. Fyne 개발 빌드는 Go 외에 C 컴파일러와 데스크톱 그래픽 개발 환경이 필요하므로 기본 콘솔 런처와 `fyne` 빌드 태그 기반 GUI 런처를 분리한다.

컨트롤 패널 역할:

- 서버 시작/중지
- IP 표시
- 포트 표시
- 접속 주소 표시
- 접속자 수 표시
- DB 상태 표시
- 최근 백업 표시
- 백업 실행
- 로그 보기
- 설정 변경
- 프로그램 종료

## 5. 웹 UI 선택

## 5.1 React를 기본으로 선택하지 않는 이유

초기 MVP의 화면은 대부분 업무용 폼과 테이블이다.

- 회원 검색
- 강좌 목록
- 신청 입력
- 신청 현황
- 추첨 결과
- 출석부 출력

이 정도는 React SPA 없이도 구현 가능하다.

React를 사용하면 다음 부담이 생긴다.

- Node.js 개발 체인
- 빌드 관리
- 상태 관리 복잡도
- API/프론트 분리 복잡도
- 초기 MVP 개발량 증가

## 5.2 Go template + HTMX를 선택하는 이유

Go template + HTMX는 서버 렌더링 기반으로 단순하게 개발할 수 있다.

장점:

- 프론트 빌드 과정 최소화
- 부분 갱신 가능
- 배포 단순
- 업무용 화면에 적합
- Go binary에 템플릿/정적 파일 embed 가능
- 유지보수 쉬움

예시 동작:

- 회원 검색 결과만 부분 갱신
- 강좌 선택 시 신청 가능 여부만 갱신
- 신청 저장 후 신청 목록만 갱신
- 추첨 결과 테이블만 갱신

## 6. SQLite를 사용하는 이유

## 6.1 포터블 노트북 모드에 적합

SQLite는 별도 DB 서버가 필요 없다.

DB 파일:

```text
data/center.db
```

장점:

- 파일 기반
- 설치 불필요
- 백업/복구 쉬움
- 포터블 배포에 적합
- 동시 접속 5대 수준에서는 충분

## 6.2 SQLite 권장 설정

```sql
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;
```

WAL 모드를 사용해 읽기와 쓰기 충돌을 줄인다.  
트랜잭션은 짧게 유지한다.

## 6.3 PostgreSQL을 MVP에서 제외하는 이유

PostgreSQL은 서버형 운영에는 좋지만 포터블 노트북 MVP에는 과하다.

문제점:

- 별도 DB 서버 실행 필요
- 포트/계정/비밀번호 관리 필요
- 설치 또는 서비스 등록 부담
- 사회복지사 담당자가 직접 켜고 끄기 어려움
- 포터블 배포 난이도 증가

추후 상시 서버 모드에서는 PostgreSQL 지원을 고려할 수 있다.

## 7. Excel 처리

추천 라이브러리:

```text
github.com/xuri/excelize/v2
```

필수 기능:

- 회원명단 가져오기
- 강좌목록 가져오기
- 신청 결과 내보내기
- 강좌별 수강자 명단 내보내기
- 추첨 결과 내보내기
- 대기자 명단 내보내기
- 출석부 생성

주의:

- 미디어 업로드는 지원하지 않는다.
- 엑셀 파일은 파싱 후 영구 저장하지 않는다.
- 내보낸 파일은 `exports/`에 저장한다.

## 8. 로깅

추천:

```text
표준 slog
또는
zerolog
```

MVP에서는 표준 `slog`를 우선 고려한다.

로그 위치:

```text
logs/app.log
```

로그 원칙:

- 개인정보 최소 기록
- 오류 원인 추적 가능
- 백업/추첨/서버 시작/중지 기록
- 사용자에게 보여줄 메시지와 개발자 로그 분리

## 9. 설정 파일

설정 파일:

```text
config.json
```

예시:

```json
{
  "app": {
    "display_name": "배움마루",
    "english_name": "Baeum-Maru",
    "mode": "portable"
  },
  "server": {
    "host": "0.0.0.0",
    "port": 18080
  },
  "database": {
    "path": "./data/center.db"
  },
  "backup": {
    "path": "./backups",
    "keep_days": 30
  },
  "ui": {
    "open_browser_on_start": true
  },
  "auth": {
    "disabled": false,
    "admin_password": "admin",
    "session_secret": "자동 생성 값",
    "session_max_age_minutes": 720
  }
}
```

`auth.admin_password`는 첫 실행 후 기관에서 정한 값으로 변경한다. `auth.disabled`를 `true`로 바꾸면 로그인 보호를 끌 수 있지만, 내부망 공유 Wi-Fi에서 운영할 때는 권장하지 않는다.

Fyne 런처 패널에서는 웹 업무 사용자를 직접 만들기보다 직원 정보와 유효 기간을 입력한 뒤 접속 코드를 발급한다. 웹 로그인은 발급된 접속 코드로 처리하고, DB에는 원문 코드가 아니라 해시와 만료/폐기 상태만 저장한다. `auth.admin_password`는 GUI 런처를 사용하지 못하는 환경의 임시 fallback으로만 둔다.

## 10. 패키징

배포 방식:

```text
BaeumMaru_Portable_v0.1.0.zip
```

포함 파일:

```text
baeum-maru.exe
README_FIRST_RUN.txt
config.json
```

첫 실행 시 자동 생성:

```text
data/
backups/
exports/
imports/
logs/
```

## 11. 제외하거나 후순위로 둔 기술

## 11.1 Python + PyInstaller

장점:

- 개발 속도 빠름
- FastAPI 사용 가능
- Excel 처리 라이브러리 풍부

후순위 이유:

- 패키징 결과물이 큼
- PyInstaller 설정 부담
- hidden import 문제
- 실행 초기 로딩 느릴 수 있음
- PyQt 등을 포함하면 배포 크기 증가
- 포터블 프로그램 품질이 애매할 수 있음

## 11.2 Electron

후순위 이유:

- 배포 용량 큼
- 런타임 무거움
- 컨트롤 패널 용도로는 과함

## 11.3 Docker

MVP 노트북 모드에서는 제외한다.

이유:

- 사회복지사 선생님이 직접 실행하기 어렵다.
- Docker Desktop 설치가 필요하다.
- 포터블 실행 조건과 맞지 않는다.

추후 상시 서버 모드에서는 Docker 배포를 지원한다.

## 12. 최종 스택 요약

```text
배움마루 MVP

Desktop Launcher:
Go console launcher by default

Desktop GUI:
Fyne launcher with `fyne` build tag

Internal Server:
Go net/http + chi

Web UI:
Go html/template + HTMX + Pico CSS or Bootstrap

Database:
SQLite WAL

Excel:
Excelize

Config:
JSON

Logging:
slog

Packaging:
Portable ZIP

Target:
Windows office laptop

Clients:
Host laptop + up to 4 LAN clients
```
