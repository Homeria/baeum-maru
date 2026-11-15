import { afterEach, describe, expect, test, vi } from "vitest";
import {
  clearSession,
  getSession,
  setSession,
  subscribeSession,
  type Session,
} from "./session";

const SESSION: Session = {
  token: "a".repeat(64),
  staff: { id: 1, name: "관리자", role: "admin" },
};

afterEach(() => {
  clearSession();
});

describe("session", () => {
  test("로그인하면 세션을 두고, 비우면 없앤다", () => {
    expect(getSession()).toBeNull();

    setSession(SESSION);
    expect(getSession()).toEqual(SESSION);

    clearSession();
    expect(getSession()).toBeNull();
  });

  test("바뀔 때마다 알리고, 이미 비었으면 알리지 않는다", () => {
    const listener = vi.fn();
    const unsubscribe = subscribeSession(listener);

    setSession(SESSION);
    clearSession();
    clearSession();
    unsubscribe();
    setSession(SESSION);

    expect(listener).toHaveBeenCalledTimes(2);
  });
});
