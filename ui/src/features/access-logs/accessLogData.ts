import type { AccessEventCode } from "../../api/generated/AccessEventCode";

/** 접속 기록에 남은 일의 화면 이름. */
export const ACCESS_EVENT_LABELS: Record<AccessEventCode, string> = {
  login: "로그인",
  login_failed: "로그인 실패",
  logout: "로그아웃",
  unlock_failed: "잠금 풀기 실패",
  recovery_used: "복구 코드 사용",
  restore: "복원",
  device_approved: "기기 승인",
  device_rejected: "기기 거절",
  device_revoked: "기기 승인 취소",
  device_approval_failed: "기기 승인 실패",
};

/** 실패라서 눈에 띄게 보여 줄 일. */
export function isFailure(event: AccessEventCode): boolean {
  return (
    event === "login_failed" ||
    event === "unlock_failed" ||
    event === "device_approval_failed"
  );
}

/** 호스트의 UTC 시각을 이 PC의 시각 `YYYY-MM-DD HH:mm:ss`로. */
export function formatOccurredAt(iso: string): string {
  const time = new Date(iso);
  const pad = (value: number) => String(value).padStart(2, "0");
  const date = `${time.getFullYear()}-${pad(time.getMonth() + 1)}-${pad(time.getDate())}`;
  const clock = `${pad(time.getHours())}:${pad(time.getMinutes())}:${pad(time.getSeconds())}`;
  return `${date} ${clock}`;
}
