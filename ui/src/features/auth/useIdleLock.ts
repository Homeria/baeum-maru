import { useEffect } from "react";
import { isIdleTooLong, lockSession } from "../../api/session";

/** 화면이 스스로 잠기는지 확인하는 간격. */
const CHECK_EVERY_MS = 15 * 1000;

/**
 * BR-K8: 마지막 요청에서 10분이 지나면 화면을 스스로 잠근다.
 *
 * 판단의 기준은 호스트다 (ADR 0014). 이것은 호스트에 물어보기 전에 미리 가리는 것이다.
 * `active`가 `false`면(로그인 전, 이미 잠김) 확인하지 않는다.
 */
export function useIdleLock(active: boolean): void {
  useEffect(() => {
    if (!active) {
      return;
    }
    const timer = window.setInterval(() => {
      if (isIdleTooLong(Date.now())) {
        lockSession();
      }
    }, CHECK_EVERY_MS);
    return () => window.clearInterval(timer);
  }, [active]);
}
