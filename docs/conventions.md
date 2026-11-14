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
- 여러 부분을 엮는 통합 테스트는 crate의 `tests/` 폴더에 둔다.

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

## 아직 정하지 않은 것

- 오류 처리 방식
- 모듈 파일 배치

뼈대 코드에서 처음 나올 때 정해서 여기에 적는다.
