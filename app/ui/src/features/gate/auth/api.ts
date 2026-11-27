import { getJson, postJson, postNoContent } from "../../../api/client";
import type { LoginRequest } from "../../../api/generated/LoginRequest";
import type { LoginResponse } from "../../../api/generated/LoginResponse";
import type { LoginStaffListResponse } from "../../../api/generated/LoginStaffListResponse";
import {
  clearSession,
  getSession,
  setSession,
  unlockSession,
} from "../../../api/session";

/** 로그인 화면에서 고를 직원. 사용 중지했거나 유효기간이 지난 계정은 빠져 있다. */
export function listLoginStaff(): Promise<LoginStaffListResponse> {
  return getJson<LoginStaffListResponse>("/api/login/staff");
}

/** PIN을 확인받고 세션을 시작한다. 성공하면 세션 증표를 메모리에 둔다. */
export async function login(request: LoginRequest): Promise<LoginResponse> {
  const response = await postJson<LoginResponse>("/api/login", request);
  setSession({ token: response.token, staff: response.staff });
  return response;
}

/** BR-K8: 같은 직원의 PIN으로 잠긴 화면을 연다. */
export async function unlock(pin: string): Promise<void> {
  await postNoContent("/api/unlock", { pin });
  unlockSession();
}

/**
 * 로그아웃한다. 호스트에 닿지 못해도 이 PC의 세션은 비운다.
 *
 * 호스트에 남은 세션은 10분 뒤 잠기고 다음 로그인 때 지워진다 (ADR 0014).
 */
export async function logout(): Promise<void> {
  if (getSession() === null) {
    return;
  }
  try {
    await postNoContent("/api/logout");
  } catch {
    // 호스트에 닿지 못해도 이 PC에서는 로그아웃한다
  }
  clearSession();
}
