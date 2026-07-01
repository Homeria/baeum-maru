# 릴리즈 전 체크리스트

## 1. 코드

- `go test ./...` 통과
- `go vet ./...` 통과
- `go build ./cmd/server` 통과
- `go build ./cmd/launcher` 통과
- `git diff --check` 통과

## 2. 패키징

- `scripts/package_windows.ps1` 로컬 실행 확인
- `Windows Package` GitHub Actions artifact 생성 확인
- ZIP 내부에 다음 항목 포함 확인
  - `baeum-maru.exe`
  - `config.json`
  - `README_FIRST_RUN.txt`
  - `data/`
  - `backups/`
  - `exports/`
  - `imports/`
  - `logs/`

## 3. 업무 흐름

- 회원 등록과 검색
- 강좌 개설 등록
- 수강신청 등록
- 중복 신청, 시간대 충돌, 최대 신청 수 제한
- 추첨 실행과 재추첨 차단
- 선정자 확정과 취소 후 대기자 승격
- 출석 회차 생성과 출석 저장
- 엑셀 출력
- 백업 생성과 복구 예약
- 감사 로그 조회

## 4. 공개 전 점검

- 실사용 DB, 백업, 엑셀 출력, 로그 파일이 포함되지 않았는지 확인
- README의 실행 방법과 패키징 방법 확인
- `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md` 존재 확인
- 이슈/PR 템플릿 존재 확인
- 스크린샷 또는 예시 데이터의 개인정보 제거

## 5. 태그 릴리즈

```powershell
git tag v0.1.0
git push origin v0.1.0
```

태그 push 후 GitHub Actions의 `Windows Package` workflow artifact를 확인한다.
