import { afterEach, describe, expect, test, vi } from "vitest";
import {
  QUIET_AFTER_MS,
  SCREEN_LOCK_MS,
  clearSession,
  getSession,
  isIdleTooLong,
  isStaffActive,
  lockSession,
  noteRequest,
  noteStaffInput,
  setSession,
  subscribeSession,
  unlockSession,
} from "./session";

const LOGIN = {
  token: "a".repeat(64),
  staff: { id: 1, name: "관리자", role: "admin" as const },
};

afterEach(() => {
  clearSession();
  vi.useRealTimers();
});

describe("session", () => {
  test("로그인하면 잠기지 않은 세션을 두고, 비우면 없앤다", () => {
    expect(getSession()).toBeNull();

    setSession(LOGIN);
    expect(getSession()).toEqual({ ...LOGIN, locked: false });

    clearSession();
    expect(getSession()).toBeNull();
  });

  test("바뀔 때마다 알리고, 이미 비었으면 알리지 않는다", () => {
    const listener = vi.fn();
    const unsubscribe = subscribeSession(listener);

    setSession(LOGIN);
    clearSession();
    clearSession();
    unsubscribe();
    setSession(LOGIN);

    expect(listener).toHaveBeenCalledTimes(2);
  });

  test("잠그고 풀 수 있다. 같은 상태로는 다시 알리지 않는다", () => {
    setSession(LOGIN);
    const listener = vi.fn();
    subscribeSession(listener);

    lockSession();
    lockSession();
    expect(getSession()?.locked).toBe(true);
    unlockSession();
    unlockSession();
    expect(getSession()?.locked).toBe(false);

    expect(listener).toHaveBeenCalledTimes(2);
  });

  test("로그인하지 않았으면 잠글 것이 없다", () => {
    lockSession();

    expect(getSession()).toBeNull();
  });

  test("마지막 요청에서 10분이 지나면 오래 쉰 것이다", () => {
    vi.useFakeTimers();
    vi.setSystemTime(0);
    setSession(LOGIN);

    vi.setSystemTime(5 * 60 * 1000);
    noteRequest();

    expect(isIdleTooLong(5 * 60 * 1000 + SCREEN_LOCK_MS - 1)).toBe(false);
    expect(isIdleTooLong(5 * 60 * 1000 + SCREEN_LOCK_MS)).toBe(true);
  });

  test("누르거나 입력한 뒤 10초 동안만 직원이 화면을 만지는 중이다", () => {
    vi.useFakeTimers();
    vi.setSystemTime(1000);

    noteStaffInput();

    expect(isStaffActive(1000 + QUIET_AFTER_MS - 1)).toBe(true);
    expect(isStaffActive(1000 + QUIET_AFTER_MS)).toBe(false);
  });
});
