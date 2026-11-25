import { afterEach, describe, expect, test, vi } from "vitest";
import { getJson, hostErrorFrom, postForFile } from "./client";
import {
  QUIET_AFTER_MS,
  clearSession,
  getSession,
  isIdleTooLong,
  noteStaffInput,
  setSession,
} from "./session";

describe("hostErrorFrom", () => {
  test("호스트 오류 모양이면 내용을 그대로 담는다", () => {
    const error = hostErrorFrom(422, {
      code: "INVALID_INPUT",
      message: "입력을 확인해 주세요",
      rule: null,
      fields: [{ field: "name", message: "이름을 입력해 주세요" }],
    });

    expect(error.status).toBe(422);
    expect(error.code).toBe("INVALID_INPUT");
    expect(error.message).toBe("입력을 확인해 주세요");
    expect(error.fields).toEqual([
      { field: "name", message: "이름을 입력해 주세요" },
    ]);
  });

  test("업무 규칙 번호를 담는다", () => {
    const error = hostErrorFrom(409, {
      code: "DUPLICATE_MEMBER_NO",
      message: "이미 있는 회원번호입니다",
      rule: "BR-M2",
      fields: [],
    });

    expect(error.rule).toBe("BR-M2");
  });

  test("어긴 신청 규칙들을 담는다", () => {
    const violation = {
      courseId: 3,
      rule: "no_time_overlap" as const,
      otherCourseId: 1,
      canOverride: true,
    };
    const error = hostErrorFrom(409, {
      code: "RULES_BROKEN",
      message: "신청 규칙을 어긴 강좌가 있습니다",
      rule: "BR-A5",
      fields: [],
      violations: [violation],
    });

    expect(error.violations).toEqual([violation]);
  });

  test("본문이 없으면 일반 문구를 쓴다", () => {
    const error = hostErrorFrom(502, null);

    expect(error.code).toBeNull();
    expect(error.message).toContain("502");
    expect(error.fields).toEqual([]);
    expect(error.violations).toEqual([]);
  });

  test("모양이 다른 본문은 호스트 오류로 보지 않는다", () => {
    const error = hostErrorFrom(500, { message: "다른 서버의 오류" });

    expect(error.code).toBeNull();
    expect(error.message).not.toBe("다른 서버의 오류");
  });
});

describe("세션 증표", () => {
  const SESSION = {
    token: "b".repeat(64),
    staff: { id: 1, name: "관리자", role: "admin" as const },
  };

  afterEach(() => {
    clearSession();
    vi.unstubAllGlobals();
  });

  function stubFetch(status: number, body: unknown) {
    const fetchMock = vi.fn(() =>
      Promise.resolve(
        new Response(JSON.stringify(body), {
          status,
          headers: { "content-type": "application/json" },
        }),
      ),
    );
    vi.stubGlobal("fetch", fetchMock);
    return fetchMock;
  }

  function sentHeaders(fetchMock: ReturnType<typeof stubFetch>) {
    const init = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    return init[1].headers as Record<string, string>;
  }

  test("로그인했으면 요청마다 증표를 싣는다", async () => {
    setSession(SESSION);
    const fetchMock = stubFetch(200, { items: [] });

    await getJson("/api/rooms");

    expect(sentHeaders(fetchMock).authorization).toBe(
      `Bearer ${SESSION.token}`,
    );
  });

  test("로그인하지 않았으면 싣지 않는다", async () => {
    const fetchMock = stubFetch(200, { needed: true });

    await getJson("/api/setup");

    expect(sentHeaders(fetchMock).authorization).toBeUndefined();
  });

  test("호스트가 로그인이 필요하다고 하면 세션을 비운다", async () => {
    setSession(SESSION);
    stubFetch(401, {
      code: "LOGIN_REQUIRED",
      message: "로그인해 주세요",
      rule: "BR-K1",
      fields: [],
    });

    await expect(getJson("/api/rooms")).rejects.toThrow("로그인해 주세요");
    expect(getSession()).toBeNull();
  });

  test("다른 오류에는 세션을 그대로 둔다", async () => {
    setSession(SESSION);
    stubFetch(403, {
      code: "NOT_ALLOWED",
      message: "이 역할로는 할 수 없는 일입니다",
      rule: "BR-K3",
      fields: [],
    });

    await expect(getJson("/api/rooms")).rejects.toThrow();
    expect(getSession()).toEqual({ ...SESSION, locked: false });
  });

  test("호스트가 잠겼다고 하면 화면을 잠근다", async () => {
    setSession(SESSION);
    stubFetch(401, {
      code: "SESSION_LOCKED",
      message: "화면이 잠겼습니다",
      rule: "BR-K8",
      fields: [],
    });

    await expect(getJson("/api/rooms")).rejects.toThrow();
    expect(getSession()?.locked).toBe(true);
  });
});

describe("조용한 요청 (BR-K8)", () => {
  afterEach(() => {
    clearSession();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  function stubOk() {
    const fetchMock = vi.fn(() =>
      Promise.resolve(new Response("{}", { status: 200 })),
    );
    vi.stubGlobal("fetch", fetchMock);
    return fetchMock;
  }

  function quietHeader(fetchMock: ReturnType<typeof stubOk>) {
    const init = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    return (init[1].headers as Record<string, string>)["x-baeum-quiet"];
  }

  test("직원이 방금 화면을 만졌으면 보통 요청이다", async () => {
    const fetchMock = stubOk();
    noteStaffInput();

    await getJson("/api/rooms");

    expect(quietHeader(fetchMock)).toBeUndefined();
  });

  test("한동안 만지지 않았으면 조용한 요청이고, 잠금 시간을 늘리지 않는다", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(0);
    noteStaffInput();
    setSession({
      token: "c".repeat(64),
      staff: { id: 1, name: "관리자", role: "admin" },
    });
    const fetchMock = stubOk();
    vi.setSystemTime(QUIET_AFTER_MS + 9 * 60 * 1000);

    await getJson("/api/rooms");

    expect(quietHeader(fetchMock)).toBe("1");
    // 로그인한 때(0)부터 10분이 지나면 잠긴다. 조용한 요청이 미루지 않았다
    expect(isIdleTooLong(10 * 60 * 1000)).toBe(true);
  });
});

describe("postForFile", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test("응답 본문을 바이트로 돌려준다", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([80, 75])))),
    );

    const bytes = await postForFile("/api/terms/1/roster-exports", {});

    expect(Array.from(bytes)).toEqual([80, 75]);
  });
});
