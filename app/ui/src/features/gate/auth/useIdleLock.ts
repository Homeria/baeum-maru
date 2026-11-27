import { useEffect } from "react";
import {
  isIdleTooLong,
  lockSession,
  noteStaffInput,
} from "../../../api/session";

/** 화면이 스스로 잠기는지 확인하는 간격. */
const CHECK_EVERY_MS = 15 * 1000;

/**
 * BR-K8: 직원이 한 마지막 요청에서 10분이 지나면 화면을 스스로 잠근다.
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
    // 직원이 화면을 만지는지 본다. 만지지 않는 동안의 요청은 잠금 시간을 늘리지 않는다
    const options = { capture: true, passive: true };
    window.addEventListener("pointerdown", noteStaffInput, options);
    window.addEventListener("keydown", noteStaffInput, options);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("pointerdown", noteStaffInput, options);
      window.removeEventListener("keydown", noteStaffInput, options);
    };
  }, [active]);
}
