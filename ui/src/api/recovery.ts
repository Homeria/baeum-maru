import { postJson } from "./client";
import type { RecoveryRequest } from "./generated/RecoveryRequest";
import type { RecoveryResponse } from "./generated/RecoveryResponse";

/**
 * 복구 코드로 관리자 PIN을 다시 정하고 새 복구 코드를 받는다 (BR-K12).
 *
 * 로그인하지 않아도 쓴다. 새 코드는 이때 한 번만 온다.
 */
export function recover(request: RecoveryRequest): Promise<RecoveryResponse> {
  return postJson<RecoveryResponse>("/api/recovery", request);
}
