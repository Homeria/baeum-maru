# 코드 규칙

코드를 쓸 때 지키는 규칙이다. 도구가 정해 주는 것은 도구에 맡기고, 여기에는 사람이 지켜야 하는 것만 적는다.

## 도구에 맡기는 것

- 줄바꿈, 들여쓰기, 띄어쓰기: Rust는 rustfmt, 화면은 Prettier
- Rust 이름: 함수·변수 `snake_case`, 타입 `CamelCase`, 상수 `SCREAMING_SNAKE_CASE` (컴파일러가 경고한다)
- TypeScript 이름: 함수·변수 `camelCase`, 부품·타입 `PascalCase`

## 이름

- 코드 식별자는 영어로 쓰고 [용어집](domain/GLOSSARY.md)의 영어 이름을 따른다. 예: 신청 `Application`, 대기 순번 `WaitlistPosition`
- 용어집에 없는 업무 용어가 필요하면 용어집에 먼저 추가한다.

## 주석 언어

- 주석과 문서 주석은 한국어로 쓴다.
- 기술 용어는 억지로 번역하지 않는다. 예: 트랜잭션, `Result`
- 화면 문구, 오류 문구, 로그 메시지도 한국어로 쓴다.

## 문서 주석

Rust는 rustdoc 관례, 화면은 TSDoc 관례를 따른다.

- 무엇인지 한두 문장으로 쓴다. 이름을 다시 말하지 않는다.
- 매개변수와 반환값을 따로 나열하지 않는다. 타입이 선언에 보인다.
- 필요할 때만 절을 쓴다.
  - `# Errors`: 오류를 돌려주는 경우와 그 오류
  - `# Panics`: 멈출 수 있는 경우
  - `# Examples`: 쓰는 예. `cargo test` 때 실제로 실행된다.
- 업무 규칙, ADR, 실험 번호는 문장 끝에 붙이지 않고 따로 한 줄로 쓴다: `BR-A6: 조건`
  - 문서 주석 안에서는 대괄호를 쓰지 않는다. 마크다운이 `[BR-A4]: 한단어`를 링크 정의로 읽어 줄이 사라진다.
- TSDoc의 `@param`은 타입만으로 뜻이 보이지 않을 때만 쓴다.

어디에 다는가:

| 위치 | 규칙 |
|---|---|
| `crates/domain`의 공개 항목(`pub`) | 모두 단다. 검사 도구가 강제한다 |
| `ui/src/components/`, `ui/src/api/`에서 내보내는 것 | 단다 |
| 앱 crate(`src-tauri`), `ui/src/features/` | 이해하기 어려운 곳에만 단다 |

```rust
/// 회원이 이 강좌에 신청할 수 있는지 확인한다.
///
/// 학기 설정에서 켠 규칙만 검사하고, 어긴 규칙을 모두 모아 돌려준다.
///
/// BR-A6: 규칙을 어겨도 사유를 적으면 예외로 받을 수 있다.
///
/// # Errors
///
/// 하나라도 어기면 [`RuleViolations`]를 돌려준다.
/// - BR-A1: 수업 시간 겹침
/// - BR-A2: 같은 과목 중복
/// - BR-A3: 최대 신청 개수 초과
pub fn check_rules(/* … */) -> Result<(), RuleViolations> { /* … */ }
```

```rust
/// 버전 문자열에서 호환 번호(앞 두 자리)를 꺼낸다.
///
/// ADR 0010: 앞 두 자리가 같으면 호환된다.
///
/// # Errors
///
/// 숫자 세 개(`0.1.0`)가 아니면 [`VersionError`]를 돌려준다.
///
/// # Examples
///
/// ```
/// use baeum_domain::Compat;
///
/// assert_eq!(Compat::parse("0.3.7").unwrap(), Compat::parse("0.3.0").unwrap());
/// ```
pub fn parse(version: &str) -> Result<Compat, VersionError> { /* … */ }
```

```tsx
/**
 * 회원 목록 한 페이지를 받는다.
 *
 * @param query 이름, 연락처 뒷자리, 회원번호 중 아무거나. 빈 글자면 전체.
 */
export function fetchMembers(query: string, page: number): Promise<MemberPage> { /* … */ }
```

```rust
// 피할 것:
/// 신청을 확인하는 함수.          (이름을 다시 말할 뿐)
/// # Arguments                    (Rust에서 쓰지 않는 칸)
/// # Returns
/// 시간이 겹치면 거절한다 (BR-A1). (규칙 번호를 문장 끝에)
```

## 인라인 주석

- 단답형으로 쓴다.
- 코드가 무엇을 하는지는 쓰지 않는다.
- 업무 규칙, ADR, 실험 번호는 별도 줄로 쓴다: `// [BR-A4]: 조건` (일반 주석은 마크다운이 아니라 대괄호를 쓴다)
- 줄 끝 주석은 쓰지 않는다.
- 주석 처리한 코드는 남기지 않는다.

```rust
// [실험 01]: 확인부터 쓰기 잠금
let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

// 앞자리 0 보존
sheet.write_string(row, 0, &member.member_no)?;

// [BR-A4]: 같은 강좌 중복 신청 금지
if existing.iter().any(|a| a.course_id == course.id) {
    return Err(ApplyError::Duplicate);
}
```

```tsx
// [실험 03]: 조합 중 Enter 무시
if (e.nativeEvent.isComposing) return;
```

```rust
// 피할 것:
// 트랜잭션을 시작한다
let mut tx = pool.begin().await?;
let count = members.len(); // 회원 수
```

## TODO

이슈 번호를 반드시 붙인다.

```rust
// TODO(#12): 초성 검색
```

## 테스트

- 업무 규칙 테스트는 이름 앞에 규칙 번호를 붙인다: `br_a1_…`. 실패하면 어느 규칙인지 바로 보이고, `cargo test br_a1`로 그 규칙만 돌릴 수 있다.
- 규칙마다 허용 예와 거부 예를 하나 이상 만든다.
- 데이터는 지어낸 것만 쓴다. 가짜인 게 바로 보이는 값을 쓴다. 예: `회원1`, `010-0000-0001`
- 단위 테스트는 같은 파일의 `#[cfg(test)] mod tests`에 둔다.
- 요청 주소부터 DB까지 거치는 테스트는 그 기능의 요청 처리 파일(예: `server/members.rs`)의 `tests`에 둔다. 라우터에 요청을 직접 보내므로(`oneshot`) 내부 모듈을 공개하지 않아도 된다.
- 여러 기능의 테스트가 함께 쓰는 도우미는 `#[cfg(test)]`로 묶어 한곳에 둔다. 요청 보내기와 임시 DB 라우터는 `server/test_support.rs`, 임시 DB 열기는 `db::open_temp`. 두 기능에서 같은 도우미가 생기면 옮긴다.
- crate의 `tests/` 폴더는 공개 API만으로 시험할 수 있을 때 쓴다.
- 화면 테스트는 Vitest로, 테스트할 파일 옆에 `이름.test.ts`로 둔다. 설명은 한국어 문장으로 쓴다: `test("빈 검색어는 싣지 않는다", …)`
- 화면은 지금 순수 함수(계산, 변환)만 테스트한다. 버튼을 누르는 화면 동작 테스트는 아직 하지 않는다.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn br_a1_rejects_overlapping_schedule() { /* … */ }

    #[test]
    fn br_a1_allows_back_to_back_schedule() { /* … */ }
}
```

## unwrap과 expect

- `unwrap()`은 쓰지 않는다. clippy `unwrap_used`가 막는다.
- `expect("무엇이 실패했는지")`는 앱을 처음 켜는 코드에서만 쓴다. 실패하면 어차피 켤 수 없는 곳이다.
- 요청 처리, 업무 규칙, DB 코드에서는 `expect`도 쓰지 않고 `?`로 넘긴다.
- 테스트와 문서 예제에서는 둘 다 쓴다.

## 오류 처리

- 업무 규칙 crate: 오류 종류를 enum으로 정의하고 `thiserror`로 만든다.
- 앱 crate: 요청 처리의 오류는 `ApiError` enum으로 모으고, axum `IntoResponse`로 JSON 응답을 만든다.
- 앱 시작 코드는 `Box<dyn Error>`로 돌려준다. `anyhow`는 쓰지 않는다.
- 내부 오류(DB, 파일)는 로그에만 남기고, 화면에는 일반 문구만 보낸다.

오류 응답 모양:

```json
{ "code": "TIME_CONFLICT", "message": "수업 시간이 겹칩니다", "rule": "BR-A1", "fields": [] }
```

- `code`: 영어 대문자. 한 번 정하면 바꾸지 않는다. 바꾸면 버전 둘째 자리를 올린다 (첫 출시 전에는 0.1.0 고정, ADR 0010).
- `message`: 직원에게 그대로 보여 줘도 되는 한국어 문장
- `rule`: 업무 규칙 위반일 때만. 아니면 `null`
- `fields`: 입력 칸이 형식에 맞지 않을 때(`INVALID_INPUT`) 틀린 칸을 모두 담는다. `{ "field": "memberNo", "message": "..." }`. 그 밖에는 빈 배열
- 상태 코드: 요청 모양이 틀림 400, 대상이 없음 404, 입력 칸이 틀림 422, 이미 있음·버전 다름 409, 내부 오류 500

## 로그와 개인정보

- 도구: `tracing`, 날짜별 파일은 `tracing-appender`
- 위치: 데이터 폴더의 `logs/`. 30일 지나면 지운다. 백업에 넣지 않는다.
- 기본 수준은 `info`, 문제를 찾을 때만 `debug`
- 회원 정보(이름, 연락처, 생년월일, 회원번호)는 로그에 쓰지 않는다. DB 내부 id를 쓴다.
- 요청 로그에는 경로만 남긴다. `?` 뒤의 검색어와 요청 본문은 남기지 않는다.
- 개인정보 필드는 디버그 출력에서 `***`로 찍히는 감싼 타입(`Personal<T>`)에 담는다. 회원이 처음 나오는 기능에서 만든다.
- 접속 기록(ADR 0003)은 로그 파일이 아니라 DB에 남긴다.

```rust
// 피할 것:
tracing::info!("신청 저장: {}", member.name);
// 이렇게:
tracing::info!(application_id = %id, "신청 저장");
```

## JSON

- 요청·응답 구조체에는 `#[serde(rename_all = "camelCase")]`를 붙인다. `member_no` → `memberNo`
- enum 값도 camelCase로 보낸다. `Waitlisted` → `"waitlisted"`
- 오류 `code`만 영어 대문자로 쓴다.

## 요청 주소

자원 중심(REST)으로 쓴다.

| 하는 일 | 모양 |
|---|---|
| 찾기·목록 | `GET /api/members?query=홍&page=2` |
| 하나 보기 | `GET /api/members/12` |
| 만들기 | `POST /api/members` |
| 고치기 | `PATCH /api/members/12` |
| 지우기 | `DELETE /api/members/12` |
| 만들기·고치기로 나타내기 어려운 동작 | 대상 아래 동사: `POST /api/courses/7/lottery` |

- 주소는 영어 복수 명사다. 여러 단어면 하이픈을 쓴다: `/api/priority-reasons`
- 이름은 용어집의 영어 이름을 따른다.
- `?` 뒤 검색 조건은 camelCase로 쓴다: `?pageSize=20`
- 목록 응답은 `{ items, total, page, pageSize }`다. `page`는 1부터, `pageSize`는 기본 20, 최대 100이다.
- 강의실처럼 많아야 수십 개인 목록은 쪽을 나누지 않고 `{ items }`로 전부 돌려준다.
- 숫자는 DB 내부 ID다. 회원번호(`member_no`)를 주소에 쓰지 않는다.
- 요청 로그에는 `?` 앞 경로만 남긴다(로그와 개인정보 참고).

## DB

| 대상 | 규칙 | 예 |
|---|---|---|
| 테이블 | 영어 복수, snake_case | `members`, `priority_reasons` |
| 컬럼 | snake_case | `member_no`, `birth_date` |
| 내부 ID | 모든 테이블에 `id INTEGER PRIMARY KEY` | |
| 다른 테이블 가리키기 | `대상_id` | `member_id`, `course_id` |
| 만든·고친 시각 | `created_at`, `updated_at` | |
| 마이그레이션 파일 | `sqlx migrate add`가 붙이는 시각 번호 | `20261115093000_create_members.sql` |

- 컬럼 이름과 Rust 필드 이름을 같게 둔다. JSON으로 나갈 때만 camelCase가 된다.
- 회원번호처럼 기관이 붙이는 번호는 내부 ID와 따로 둔다.
- ADR 0005: 한 번 배포한 마이그레이션 파일은 고치지 않는다. 바꿀 것이 있으면 새 마이그레이션을 더한다.

## 날짜와 시간

- 라이브러리는 `chrono`를 쓴다. sqlx, ts-rs, serde가 모두 지원한다.
- DB에는 ISO 8601 글자로 저장한다. 글자 순서가 시간 순서와 같다.

| 값 | Rust 타입 | 저장 예 |
|---|---|---|
| 날짜만 (생년월일) | `NaiveDate` | `1950-03-01` |
| 시각 (접수 시각) | `DateTime<Utc>` | `2026-11-15T00:30:00.123Z` |

- 시각은 UTC로 저장하고 화면에서 한국 시간으로 바꿔 보여 준다. 바꾸는 곳은 화면의 날짜 표시 함수 한 곳이다.
- 시각은 밀리초까지 저장한다. 같은 밀리초면 내부 ID가 작은 쪽이 먼저다.
- 날짜만 있는 값에는 시간대를 붙이지 않는다.

## 모듈 파일 배치

- `server.rs` + `server/` 폴더 방식을 쓴다. `mod.rs`는 쓰지 않는다. clippy `mod_module_files`가 막는다.
- 줄 수가 아니라 하는 일 단위로 나눈다. 같은 파일의 테스트는 길이에 세지 않는다.
- 기능 하나(회원, 강좌, 신청…)는 세 곳에 나뉜다.

| 곳 | 하는 일 |
|---|---|
| `crates/domain/src/member.rs` | 업무 규칙과 확인. DB와 HTTP를 모른다 |
| `src-tauri/src/db/members.rs` | SQL |
| `src-tauri/src/server/members.rs` | 요청 처리 함수. 확인 → 저장 순서를 엮는다 |
| `src-tauri/src/server/members/json.rs` | 요청·응답 JSON 타입과 변환 |

- 요청 처리 함수는 요청 주소 표의 순서(찾기, 보기, 만들기, 고치기, 지우기)로 둔다.
- 한 요청이 여러 테이블을 한 트랜잭션으로 다루게 되면(접수 등) 그때 순서를 엮는 층을 따로 둘지 정한다.

```text
src-tauri/src/
├─ main.rs
├─ db.rs
├─ db/
│  └─ members.rs
├─ server.rs
└─ server/
   ├─ error.rs
   ├─ version.rs
   ├─ members.rs
   └─ members/
      └─ json.rs
```

## 화면 파일

| 대상 | 규칙 | 예 |
|---|---|---|
| 부품 파일 | `PascalCase`, 부품 이름과 같게 | `MemberTable.tsx`, `MemberTable.module.css` |
| 훅, 그 밖의 파일 | `camelCase` | `useHostVersion.ts`, `client.ts` |
| 폴더 | 소문자 | `components/`, `features/host/` |
| CSS 클래스 | `camelCase` | `.rowSelected` → `styles.rowSelected` |

- 이름 붙은 export만 쓴다. `export default`는 쓰지 않는다. ESLint가 막는다.
  - 도구가 default를 요구하는 설정 파일(`vite.config.ts`, `eslint.config.js`)만 예외다.
- import할 때 파일 이름의 대소문자가 틀리면 TypeScript가 오류를 낸다. Windows는 대소문자를 가리지 않아 놓치기 쉽다.
- 같은 폴더에 대소문자와 확장자만 다른 파일을 두지 않는다. `MemberForm.tsx` 옆의 계산 파일은 `memberForm.ts`가 아니라 `memberFormData.ts`처럼 이름을 다르게 짓는다. Windows는 둘을 같은 파일로 봐서 Windows 빌드만 깨진다. `just check`가 막는다.
- CSS Modules가 클래스 이름을 파일마다 바꿔 주므로 BEM처럼 긴 이름은 쓰지 않는다.

```tsx
import styles from "./Tabs.module.css";

export function Tabs() {
  return <div className={styles.tabList}>…</div>;
}
```

`*.module.css`의 `import styles`는 default import지만 Vite가 만들어 주는 것이라 괜찮다. 막는 것은 우리 파일의 default export다.
