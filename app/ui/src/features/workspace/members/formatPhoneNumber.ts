/**
 * 숫자만 있는 연락처에 하이픈을 넣는다. 모르는 모양이면 그대로 돌려준다.
 *
 * - 서울(02): `02-123-4567`, `02-1234-5678`
 * - 그 밖의 10자리: `032-555-6666`
 * - 11자리: `010-1234-5678`
 */
export function formatPhoneNumber(digits: string): string {
  if (!/^0\d+$/.test(digits)) {
    return digits;
  }
  if (digits.startsWith("02")) {
    if (digits.length === 9) {
      return join(digits, 2, 5);
    }
    if (digits.length === 10) {
      return join(digits, 2, 6);
    }
    return digits;
  }
  if (digits.length === 10) {
    return join(digits, 3, 6);
  }
  if (digits.length === 11) {
    return join(digits, 3, 7);
  }
  return digits;
}

function join(digits: string, first: number, second: number): string {
  return [
    digits.slice(0, first),
    digits.slice(first, second),
    digits.slice(second),
  ].join("-");
}
