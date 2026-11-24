import { afterEach, describe, expect, test, vi } from "vitest";
import { receiveChanges, takeEvents } from "./changes";

describe("takeEvents", () => {
  test("빈 줄로 나뉜 글을 꺼내고 덜 받은 뒷부분은 남긴다", () => {
    const { events, rest } = takeEvents(
      'event: change\ndata: "member"\n\nevent: change\ndata: "ro',
    );

    expect(events).toEqual([{ event: "change", data: '"member"' }]);
    expect(rest).toBe('event: change\ndata: "ro');
  });

  test("연결 확인 신호는 건너뛰고 CRLF 줄바꿈도 읽는다", () => {
    const { events, rest } = takeEvents(
      ':\r\n\r\nevent: change\r\ndata: "term"\r\n\r\n',
    );

    expect(events).toEqual([{ event: "change", data: '"term"' }]);
    expect(rest).toBe("");
  });
});

describe("receiveChanges", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  /** 글자 조각들을 차례로 흘려보내는 응답. */
  function streamOf(chunks: string[]): Response {
    const encoder = new TextEncoder();
    const body = new ReadableStream<Uint8Array>({
      start(controller) {
        for (const chunk of chunks) {
          controller.enqueue(encoder.encode(chunk));
        }
        controller.close();
      },
    });
    return new Response(body, { status: 200 });
  }

  test("연결되면 알리고, 조각으로 나뉘어 와도 알림을 종류로 넘긴다", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() =>
        Promise.resolve(
          streamOf([
            'event: change\ndata: "appli',
            'cation"\n\n:\n\nevent: change\ndata: "member"\n\n',
          ]),
        ),
      ),
    );
    const calls: string[] = [];

    await receiveChanges(
      new AbortController().signal,
      () => calls.push("open"),
      (kind) => calls.push(kind),
    );

    expect(calls).toEqual(["open", "application", "member"]);
  });

  test("알림 연결은 조용한 요청으로 연다", async () => {
    const fetchMock = vi.fn(() => Promise.resolve(streamOf([])));
    vi.stubGlobal("fetch", fetchMock);

    await receiveChanges(
      new AbortController().signal,
      () => undefined,
      () => undefined,
    );

    const init = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(init[0]).toMatch(/\/api\/changes$/);
    expect((init[1].headers as Record<string, string>)["x-baeum-quiet"]).toBe(
      "1",
    );
  });
});
