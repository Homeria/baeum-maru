# 배움마루 파일구조 설계

## 1. 저장소 루트 구조

```text
baeum-maru/
├─ cmd/
│  ├─ launcher/
│  │  └─ main.go
│  └─ server/
│     └─ main.go
│
├─ internal/
│  ├─ app/
│  ├─ launcher/
│  ├─ server/
│  ├─ config/
│  ├─ database/
│  ├─ migration/
│  ├─ backup/
│  ├─ logging/
│  ├─ domain/
│  ├─ service/
│  ├─ repository/
│  ├─ web/
│  └─ util/
│
├─ web/
│  ├─ templates/
│  ├─ static/
│  └─ htmx/
│
├─ migrations/
├─ docs/
├─ scripts/
├─ build/
├─ testdata/
├─ go.mod
├─ go.sum
├─ README.md
└─ .gitignore
```

현재 저장소에는 `LICENSE`와 `NOTICE`를 포함하며, 비상업 사용과 출처 고지 정책을 명시한다.

## 2. cmd 구조

## 2.1 cmd/launcher

Windows 포터블 런처 진입점이다.

```text
cmd/launcher/main.go
```

역할:

- 포터블 실행 파일 진입점
- 설정 로드
- 런타임 조립
- HTTP 서버 시작
- 브라우저 열기
- 종료 신호 처리

Fyne GUI 제어는 후속 사용성 개선 후보이다.

## 2.2 cmd/server

추후 상시 서버 모드 또는 Docker 배포용 진입점이다.

```text
cmd/server/main.go
```

역할:

- GUI 없이 HTTP 서버 실행
- CLI 플래그로 설정 지정
- Linux/Docker 배포에 사용

MVP에서는 우선순위를 낮게 두되, 구조만 미리 잡아둔다.

## 3. internal 구조

## 3.1 internal/app

애플리케이션 조립 계층.

```text
internal/app/
├─ app.go
└─ dependencies.go
```

역할:

- config 로드
- DB 연결
- repository/service/web handler 조립
- server 인스턴스 생성

## 3.2 internal/launcher

런처 관련 코드.

```text
internal/launcher/
└─ doc.go
```

역할:

- 현재는 런처 패키지 자리만 유지
- 후속 GUI 런처가 필요할 때 창, 상태, 액션 코드를 추가

## 3.3 internal/server

내장 HTTP 서버 제어.

```text
internal/server/
├─ server.go
├─ router.go
├─ lifecycle.go
└─ clients.go
```

역할:

- HTTP 서버 생성
- 라우터 등록
- 서버 시작/중지
- graceful shutdown
- 접속자 heartbeat 상태 관리

## 3.4 internal/config

설정 파일 처리.

```text
internal/config/
├─ config.go
├─ loader.go
└─ defaults.go
```

역할:

- `config.json` 읽기
- 기본값 생성
- 설정 저장
- 표시명, 포트, DB 경로, 백업 경로 관리

## 3.5 internal/database

DB 연결과 SQLite 설정.

```text
internal/database/
├─ db.go
├─ sqlite.go
└─ tx.go
```

역할:

- SQLite 연결
- WAL 모드 설정
- busy timeout 설정
- foreign key 활성화
- 트랜잭션 헬퍼 제공

## 3.6 internal/migration

DB 마이그레이션.

```text
internal/migration/
├─ migration.go
└─ runner.go
```

역할:

- DB schema version 관리
- 첫 실행 시 테이블 생성
- 버전별 마이그레이션 실행

## 3.7 internal/backup

백업/복구 기능.

```text
internal/backup/
├─ backup.go
├─ restore.go
├─ naming.go
└─ cleanup.go
```

역할:

- SQLite 안전 백업
- 백업 파일명 생성
- 복구
- 최근 백업 조회
- 추후 암호화/오래된 백업 삭제 확장

## 3.8 internal/logging

로그 초기화.

```text
internal/logging/
├─ logger.go
└─ file.go
```

역할:

- 로그 파일 생성
- slog 설정
- 사용자용 오류와 개발자용 로그 분리

## 3.9 internal/domain

도메인 모델.

```text
internal/domain/
├─ member.go
├─ course.go
├─ registration.go
├─ lottery.go
├─ waitlist.go
├─ attendance.go
├─ user.go
└─ settings.go
```

역할:

- 핵심 엔티티 정의
- 상태 enum 정의
- 도메인 규칙 일부 정의

## 3.10 internal/repository

DB 접근 계층.

```text
internal/repository/
├─ member_repository.go
├─ course_repository.go
├─ registration_repository.go
├─ lottery_repository.go
├─ attendance_repository.go
├─ settings_repository.go
└─ audit_repository.go
```

역할:

- SQL 실행
- CRUD
- 조회 쿼리
- 트랜잭션 내 저장 처리

MVP에서는 `database/sql`과 직접 SQL을 우선 사용한다.

## 3.11 internal/service

비즈니스 로직.

```text
internal/service/
├─ member_service.go
├─ course_service.go
├─ registration_service.go
├─ rule_service.go
├─ lottery_service.go
├─ waitlist_service.go
├─ attendance_service.go
├─ excel_service.go
└─ report_service.go
```

역할:

- 신청 제한 검사
- 추첨 실행
- 대기자 생성
- 엑셀 import/export
- 출석부 생성
- 도메인 로직 처리

## 3.12 internal/web

HTTP handler 계층.

```text
internal/web/
├─ handler.go
├─ middleware.go
├─ routes.go
├─ member_handler.go
├─ course_handler.go
├─ registration_handler.go
├─ lottery_handler.go
├─ report_handler.go
├─ export_handler.go
└─ heartbeat_handler.go
```

역할:

- HTTP 요청 처리
- 템플릿 렌더링
- HTMX 요청 처리
- 파일 다운로드 응답
- heartbeat 처리

## 3.13 internal/util

공통 유틸리티.

```text
internal/util/
├─ network.go
├─ browser.go
├─ filesystem.go
├─ time.go
└─ validation.go
```

역할:

- 내부 IP 조회
- 브라우저 열기
- 폴더 열기
- 날짜/시간 포맷
- 입력값 검증

## 4. web 구조

## 4.1 templates

```text
web/templates/
├─ layout/
│  ├─ base.html
│  └─ print.html
├─ pages/
│  ├─ dashboard.html
│  ├─ members.html
│  ├─ member_detail.html
│  ├─ courses.html
│  ├─ registration.html
│  ├─ lottery.html
│  ├─ waitlist.html
│  ├─ reports.html
│  └─ settings.html
└─ partials/
   ├─ member_search_results.html
   ├─ course_table.html
   ├─ registration_form.html
   ├─ registration_list.html
   ├─ lottery_result_table.html
   └─ alert.html
```

## 4.2 static

```text
web/static/
├─ css/
│  └─ app.css
├─ js/
│  ├─ app.js
│  └─ heartbeat.js
└─ img/
   └─ logo.svg
```

## 5. migrations 구조

```text
migrations/
├─ 001_init.sql
├─ 002_add_lottery.sql
└─ 003_add_attendance.sql
```

초기에는 SQL 파일 기반으로 두거나 Go 코드에 embed할 수 있다.

## 6. docs 구조

```text
docs/
├─ 01_project_summary.md
├─ 02_requirements.md
├─ 03_tech_stack.md
├─ 04_runtime_architecture.md
├─ 05_file_structure.md
├─ 06_development_plan.md
├─ 07_mvp_scope.md
├─ 08_data_model.md
├─ 09_screen_flows.md
├─ 10_business_rules.md
├─ 11_quality_checklist.md
├─ 12_codex_task_breakdown.md
└─ 13_current_state.md
```

## 7. scripts 구조

```text
scripts/
├─ package_windows.ps1
└─ README.md
```

역할:

- Windows exe 빌드
- portable ZIP 생성
- 개발 실행
- 테스트 실행
- 추후 Linux/Docker 빌드

## 8. 포터블 배포 구조

사용자에게 제공되는 ZIP 구조.

```text
BaeumMaru_Portable_v0.1.0/
├─ baeum-maru.exe
├─ README_FIRST_RUN.txt
└─ config.json
```

첫 실행 후 자동 생성:

```text
BaeumMaru_Portable_v0.1.0/
├─ baeum-maru.exe
├─ README_FIRST_RUN.txt
├─ config.json
├─ data/
│  └─ center.db
├─ backups/
├─ exports/
├─ imports/
└─ logs/
   └─ app.log
```

## 9. .gitignore 기준

```gitignore
# build
/build/
/dist/
*.exe
*.zip

# runtime data
/data/
/backups/
/exports/
/imports/
/logs/

# config with local secrets
config.local.json

# OS
.DS_Store
Thumbs.db

# Go
*.test
coverage.out
```

## 10. 이름 변경 대비

프로젝트 표시명은 변경될 수 있으므로 다음 원칙을 지킨다.

좋은 예:

```text
internal/app
internal/server
internal/launcher
internal/course
internal/registration
APP_DISPLAY_NAME = "배움마루"
```

피해야 할 예:

```text
internal/baeummaru
/api/baeum-maru/...
baeum_maru_members
```

표시명, 실행 파일명, README 이름은 나중에 바꿀 수 있지만, 내부 구조는 도메인 중심으로 유지한다.
