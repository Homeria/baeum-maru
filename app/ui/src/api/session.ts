import type { SignedInStaffResponse } from "./generated/SignedInStaffResponse";

/** 로그인한 상태: 세션 증표, 로그인한 직원, 화면 잠금. */
export interface Session {
  /** 요청마다 `Authorization: Bearer <증표>`로 싣는다. */
  token: string;
  staff: SignedInStaffResponse;
  /** BR-K8: 잠겼으면 같은 직원의 PIN으로 열어야 다른 요청이 된다. */
  locked: boolean;
}

/** BR-K8: 마지막 요청에서 이만큼 지나면 화면을 잠근다. 호스트의 값과 같다. */
export const SCREEN_LOCK_MS = 10 * 60 * 1000;

// ADR 0014: 증표는 메모리에만 둔다. 앱을 끄면 사라지고 다시 로그인한다
let current: Session | null = null;
// 마지막으로 호스트가 세션을 받아 준 시각(밀리초). 다시 그릴 일이 아니라 따로 둔다
let lastRequestAt = 0;
const listeners = new Set<() => void>();

/** 지금 세션. 로그인하지 않았으면 `null`. */
export function getSession(): Session | null {
  return current;
}

/** 로그인했다. */
export function setSession(session: Omit<Session, "locked">): void {
  current = { ...session, locked: false };
  lastRequestAt = Date.now();
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

/** 화면을 잠근다. 로그인하지 않았거나 이미 잠겼으면 아무것도 하지 않는다. */
export function lockSession(): void {
  if (current === null || current.locked) {
    return;
  }
  current = { ...current, locked: true };
  notify();
}

/** PIN으로 잠금을 풀었다. */
export function unlockSession(): void {
  if (current === null || !current.locked) {
    return;
  }
  current = { ...current, locked: false };
  lastRequestAt = Date.now();
  notify();
}

/** 호스트가 세션을 받아 줬다. 화면이 스스로 잠그는 시각을 미룬다. */
export function noteRequest(): void {
  lastRequestAt = Date.now();
}

/**
 * BR-K8: 직원이 마지막으로 누르거나 입력한 뒤 이만큼 지나면, 그때 나가는 요청은 화면이 스스로 한
 * 요청으로 본다(예: 변경 알림 때문에 다시 받기).
 */
export const QUIET_AFTER_MS = 10 * 1000;

// 직원이 마지막으로 누르거나 입력한 시각(밀리초)
let lastInputAt = 0;

/** 직원이 화면을 누르거나 입력했다. */
export function noteStaffInput(): void {
  lastInputAt = Date.now();
}

/** 직원이 방금 화면을 만졌는지. 아니면 요청을 조용한 요청으로 보낸다. */
export function isStaffActive(now: number): boolean {
  return now - lastInputAt < QUIET_AFTER_MS;
}

/** 마지막 요청에서 10분이 지났는지. 화면이 스스로 잠글지 정할 때 쓴다. */
export function isIdleTooLong(now: number): boolean {
  return now - lastRequestAt >= SCREEN_LOCK_MS;
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
