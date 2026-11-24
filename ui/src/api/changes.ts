import { HostError, openStream } from "./client";
import type { ChangeKind } from "./generated/ChangeKind";

/** SSE 글 하나. */
export interface StreamEvent {
  event: string;
  data: string;
}

/**
 * 지금까지 받은 SSE 글자에서 다 받은 글들을 꺼낸다. 덜 받은 뒷부분은 `rest`로 돌려준다.
 *
 * 글은 빈 줄로 나뉜다. `:`로 시작하는 줄(연결 확인 신호)은 건너뛴다.
 */
export function takeEvents(buffer: string): {
  events: StreamEvent[];
  rest: string;
} {
  const blocks = buffer.replace(/\r\n/g, "\n").split("\n\n");
  const rest = blocks.pop() ?? "";
  const events: StreamEvent[] = [];
  for (const block of blocks) {
    let event = "message";
    const data: string[] = [];
    for (const line of block.split("\n")) {
      if (line.startsWith("event:")) {
        event = line.slice("event:".length).trim();
      } else if (line.startsWith("data:")) {
        data.push(line.slice("data:".length).trimStart());
      }
    }
    if (data.length > 0) {
      events.push({ event, data: data.join("\n") });
    }
  }
  return { events, rest };
}

/**
 * 호스트의 변경 알림(ADR 0008)을 받는다. 연결이 끝나면 돌아온다.
 *
 * 연결되면 `onOpen`, 알림마다 `onChange`를 부른다. 호스트에 닿지 못하거나 거절되면
 * {@link HostError}를, 받는 중에 끊기면 그 오류를 던진다.
 */
export async function receiveChanges(
  signal: AbortSignal,
  onOpen: () => void,
  onChange: (kind: ChangeKind) => void,
): Promise<void> {
  const response = await openStream("/api/changes", signal);
  if (response.body === null) {
    throw new HostError(0, "변경 알림을 받지 못했습니다");
  }
  onOpen();
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  for (;;) {
    // 연결이 끊기거나 signal로 그만두면 여기서 던진다. 부른 쪽이 signal을 보고 가린다
    const { done, value } = await reader.read();
    if (done) {
      return;
    }
    buffer += decoder.decode(value, { stream: true });
    const taken = takeEvents(buffer);
    buffer = taken.rest;
    for (const { event, data } of taken.events) {
      if (event === "change") {
        onChange(JSON.parse(data) as ChangeKind);
      }
    }
  }
}
