# 프로토타입 개발 계획

## 원칙

- MVP를 빠르게 끝내는 것이 목표가 아니다.
- 업무 규칙을 유지한 채 실제 운영 화면과 배포 구조를 검증한다.
- 큰 전환은 작고 검증 가능한 브랜치로 나눈다.
- 사용자 데이터가 아직 없으므로, 스키마 오류는 지금 바로 바로잡는다.

브랜치별 전체 순서와 수동 검증 지점은 `18_prototype_branch_roadmap.md`를 단일 기준으로 사용한다. 이 문서는 단계의 목적을 설명하고, 실제 다음 브랜치 판단은 로드맵 문서를 따른다.

## 단계 0: 기준선 정리

- 현재 `develop`을 `main`에 반영해 기능 MVP 기준점을 보존한다.
- 프로토타입 브랜치 로드맵과 현재/목표 상태를 문서에 확정한다.
- 기존 행동을 characterization test로 고정한 후 구조를 변경한다.

## 단계 1: 애플리케이션 경계 정리

- Fyne에 묶인 서버 제어, 네트워크 주소 탐색, 로그 구독을 `internal/launcher`로 추출한다.
- Go template handler에서 재사용 가능한 command/query DTO를 서비스 경계에 둔다.
- Huma v2를 `net/http` 위에 도입하고 `/api/v1`의 OpenAPI, 인증, 오류 형식, 페이지네이션, 필터, 권한 규약을 정의한다.
- 공간의 층 참조 모델을 확정하고 필요하면 `locations.building_floor_id`로 정리한다.

## 단계 2: React 웹 기반

- `frontend/web`에 React/Vite/TypeScript를 구성한다.
- Go 빌드에 웹 정적 자산 embed 단계를 연결한다.
- 로그인, 앱 shell, 권한별 메뉴, API client, SSE query invalidation을 만든다.
- Go template 화면은 즉시 삭제하지 않고 전환 기간의 비교 기준으로 둔다.

## 단계 3: 핵심 업무 화면 재설계

우선순위는 다음과 같다.

1. 접수: 회원 검색/신규 등록/수정과 다중 강좌 선택을 한 흐름으로 처리
2. 회원 관리: 검색, 상세, 수정, 신청 이력
3. 강좌 운영: 회차, 분류, 과목, 강사, 공간, 복수 시간표, 정원 타입
4. 신청 현황: 검색, 상태 변경, 충돌/정원 정보, 실시간 갱신
5. 추첨, 출석, Excel, 백업 화면

## 단계 4: Wails 런처 프로토타입

- `spike/wails-launcher-prototype`에서 WebView2, 한글 입력, 탭 상태 유지, 실행 파일 크기를 검증한다.
- 통과 기준: 서버 시작/중지, 주소 표시, 접속 코드 목록, 한글 입력, 로그 이벤트가 실제 Windows 장비에서 동작한다.
- 이후 `frontend/launcher`과 `cmd/launcher`를 Wails로 교체한다. Fyne는 전환 완료까지 유지한다.
- Wails와 별개로 콘솔 서버 fallback 실행 경로를 유지하고, WebView2가 없는 PC에서도 브라우저 업무가 가능한지 확인한다.

## 단계 5: 보안과 동시성

- HTTPS 인증서 생성/설정과 안전한 쿠키를 도입한다.
- CSRF, 로그인 실패 제한, 보안 헤더, 파일 업로드 검증을 추가한다.
- 2~5개 브라우저 동시 접수, 동일 회원 동시 신청, 서버 재시작, 이벤트 재연결을 자동/수동 검증한다.

## 단계 6: 운영 검증과 릴리즈

- 회원 1,000명, 강좌 50개, 신청 3,000건 수준의 더미 데이터로 흐름을 점검한다.
- 사무용 Windows 노트북에서 Wails/WebView2와 포터블 패키지를 확인한다.
- Windows CI에서 React 빌드, Huma API contract 검사, Wails 빌드, Go test/race, WebView2/fallback 패키지 smoke test를 실행한다.
- 사용자 가이드와 장애 대응 가이드를 작성한다.
