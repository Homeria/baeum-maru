import type { SignedInStaffResponse } from "./generated/SignedInStaffResponse";

/** 로그인한 상태: 세션 증표와 로그인한 직원. */
export interface Session {
  /** 요청마다 `Authorization: Bearer <증표>`로 싣는다. */
  token: string;
  staff: SignedInStaffResponse;
}

// ADR 0014: 증표는 메모리에만 둔다. 앱을 끄면 사라지고 다시 로그인한다
let current: Session | null = null;
const listeners = new Set<() => void>();

/** 지금 세션. 로그인하지 않았으면 `null`. */
export function getSession(): Session | null {
  return current;
}

/** 로그인했다. */
export function setSession(session: Session): void {
  current = session;
  notify();
}

/** 로그아웃했거나 호스트가 세션을 끝냈다. 이미 비었으면 아무것도 하지 않는다. */
export function clearSession(): void {
  if (current === null) {
    return;
  }
  current = null;
  notify();
}

/** 세션이 바뀔 때마다 `listener`를 부른다. 돌려준 함수를 부르면 그만 부른다. */
export function subscribeSession(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function notify() {
  for (const listener of listeners) {
    listener();
  }
}
