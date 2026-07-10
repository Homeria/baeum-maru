# 작업 단위 가이드

이 문서는 이후 브랜치를 작고 검증 가능한 단위로 나누기 위한 기준이다. 이전 MVP 작업은 대부분 완료됐으므로, 새 작업은 프로토타입 전환에 맞춘다.

## 우선 작업 순서

1. `refactor/web-layout-shell` 마감
2. `docs/prototype-architecture` 문서 기준 확정
3. `refactor/launcher-core` Fyne 의존성에서 런처 제어 로직 분리
4. `refactor/api-boundary` Huma v2 `/api/v1` DTO, OpenAPI, 오류, 인증/권한 경계
5. `spike/wails-launcher-prototype` Windows WebView2 검증
6. `feat/react-web-foundation` React/Vite 앱 shell, 로그인, API client, SSE
7. `feat/reception-workflow-redesign` 회원+다중 강좌 접수 흐름
8. `feat/course-operations-ui` 강좌 운영 기준 데이터와 복수 시간표 UI
9. `feat/wails-launcher` 런처 전환
10. `test/prototype-operation-simulation` 다중 브라우저/성능/복구 리허설

## 브랜치 하나의 기준

- 하나의 사용자 가치 또는 하나의 아키텍처 경계만 바꾼다.
- DB 변경이 있으면 스키마 문서, 서비스 테스트, UI 영향을 함께 점검한다.
- UI 변경은 최소한 데스크톱과 좁은 화면에서 수동 확인한다.
- API 변경은 성공, 검증 오류, 권한 오류, 동시성 충돌 응답을 테스트한다.
- Fyne/Wails/React 전환 중에는 이전 구현을 단순히 지우지 않고 대체 경로가 검증된 뒤 제거한다.

## Codex 요청 예시

```text
docs 기준으로 refactor/api-boundary를 진행해줘.
Go template handler는 유지하고, Huma v2로 React가 사용할 회원 조회 API만 먼저 추가해줘.
권한/오류 응답 테스트와 go test ./...를 포함해줘.
```

```text
spike/wails-launcher-prototype을 진행해줘.
기존 Fyne 런처는 건드리지 말고, 서버 상태와 한글 입력만 검증 가능한 독립 프로토타입으로 만들어줘.
Windows 빌드 결과와 WebView2 의존성을 확인해줘.
```
