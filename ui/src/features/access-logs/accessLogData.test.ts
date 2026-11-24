import { describe, expect, test } from "vitest";
import { formatOccurredAt, isFailure } from "./accessLogData";

describe("formatOccurredAt", () => {
  test("이 PC의 시각으로 보여 준다", () => {
    // 이 PC의 2026-11-15 09:05:03을 호스트가 UTC로 보낸 것
    const iso = new Date(2026, 10, 15, 9, 5, 3).toISOString();

    expect(formatOccurredAt(iso)).toBe("2026-11-15 09:05:03");
  });
});

describe("isFailure", () => {
  test("로그인, 잠금 풀기, 기기 승인의 PIN 실패만", () => {
    expect(isFailure("login_failed")).toBe(true);
    expect(isFailure("unlock_failed")).toBe(true);
    expect(isFailure("device_approval_failed")).toBe(true);
    expect(isFailure("device_rejected")).toBe(false);
    expect(isFailure("login")).toBe(false);
    expect(isFailure("recovery_used")).toBe(false);
  });
});
