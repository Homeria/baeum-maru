/** 두 번 넣은 PIN이 다를 때의 문구. 같으면 `null`. PIN 모양은 호스트가 확인한다. */
export function pinConfirmError(
  pin: string,
  pinConfirm: string,
): string | null {
  return pin === pinConfirm ? null : "위에 넣은 PIN과 다릅니다";
}
