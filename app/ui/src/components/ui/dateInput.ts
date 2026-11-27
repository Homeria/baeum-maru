const NON_HYPHEN_DATE_PATTERN =
  /^\s*(\d{4})[./\s년]+(\d{1,2})[./\s월]+(\d{1,2})\s*일?\s*$/;

/**
 * 사용자가 입력하거나 붙여넣은 날짜 글자를 `YYYY-MM-DD` 형식으로 정리한다.
 *
 * - `19550315`처럼 숫자 8자리를 연속 입력하면 `1955-03-15`로 하이픈을 자동 삽입한다.
 * - `1955.3.5`, `1955/03/15`, `1955년 3월 15일` 같은 구분자 날짜도 `YYYY-MM-DD`로 맞춘다.
 * - 하이픈 형식(`1955-03-15`)에서 백스페이스로 지울 때는 숫자만 추려 다시 묶어
 *   끝자리나 하이픈에 걸리지 않고 자연스럽게 지워지게 한다.
 */
export function formatDateInput(raw: string): string {
  const separated = NON_HYPHEN_DATE_PATTERN.exec(raw);
  if (separated) {
    const [, year, month, day] = separated;
    return `${year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`;
  }
  const digits = raw.replace(/\D/g, "").slice(0, 8);
  if (digits.length <= 4) {
    return digits;
  }
  if (digits.length <= 6) {
    return `${digits.slice(0, 4)}-${digits.slice(4)}`;
  }
  return `${digits.slice(0, 4)}-${digits.slice(4, 6)}-${digits.slice(6)}`;
}
