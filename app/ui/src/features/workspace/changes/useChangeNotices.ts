import { useEffect, useState } from "react";
import { useQueryClient, type QueryClient } from "@tanstack/react-query";
import { HostError } from "../../../api/client";
import { receiveChanges } from "../../../api/changes";
import { CHANGE_KEYS } from "./changeKeys";

/** 알림 연결의 상태. */
export type ChangeConnection = "connecting" | "open" | "lost";

/** 끊긴 뒤 다시 연결하기까지 기다리는 시간. 실패할수록 길게, 마지막 값에서 멈춘다. */
const RETRY_AFTER_MS = [1000, 2000, 5000, 10000] as const;

/**
 * 호스트의 변경 알림(ADR 0008)을 받아 바뀐 목록을 다시 받는다.
 *
 * - `active`가 `false`면(로그인 전, 화면 잠김) 연결하지 않고, 열려 있던 연결은 닫는다.
 * - 끊기면 `"lost"`를 돌려주고 알아서 다시 연결한다. 다시 붙으면 보던 데이터를 전부 다시 받는다.
 * - 입력 중인 칸은 각 화면의 상태라 건드려지지 않는다.
 */
export function useChangeNotices(active: boolean): ChangeConnection {
  const queryClient = useQueryClient();
  const [connection, setConnection] = useState<ChangeConnection>("connecting");

  useEffect(() => {
    if (!active) {
      return;
    }
    const controller = new AbortController();
    void keepReceiving(queryClient, controller.signal, setConnection);
    return () => {
      controller.abort();
      setConnection("connecting");
    };
  }, [active, queryClient]);

  return active ? connection : "connecting";
}

async function keepReceiving(
  queryClient: QueryClient,
  signal: AbortSignal,
  setConnection: (connection: ChangeConnection) => void,
): Promise<void> {
  let failures = 0;
  while (!signal.aborted) {
    try {
      await receiveChanges(
        signal,
        () => {
          if (failures > 0) {
            // 끊긴 동안 놓친 알림이 있을 수 있다 (ADR 0008)
            void queryClient.invalidateQueries();
          }
          failures = 0;
          setConnection("open");
        },
        (kind) => {
          for (const key of CHANGE_KEYS[kind]) {
            void queryClient.invalidateQueries({ queryKey: [key] });
          }
        },
      );
    } catch (failure) {
      // 세션이 끝났거나 잠겼다. 요청 쪽이 이미 로그인·잠금 화면으로 돌렸다
      if (failure instanceof HostError && failure.status === 401) {
        return;
      }
    }
    if (signal.aborted) {
      return;
    }
    // 연결이 끝났다. 호스트가 꺼졌거나, 알림이 밀려 호스트가 끊었다
    failures += 1;
    setConnection("lost");
    const delay = RETRY_AFTER_MS[Math.min(failures, RETRY_AFTER_MS.length) - 1];
    await sleep(delay, signal);
  }
}

/** `ms`만큼 기다린다. `signal`로 그만두면 바로 돌아온다. */
function sleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    const timer = window.setTimeout(resolve, ms);
    signal.addEventListener(
      "abort",
      () => {
        window.clearTimeout(timer);
        resolve();
      },
      { once: true },
    );
  });
}
