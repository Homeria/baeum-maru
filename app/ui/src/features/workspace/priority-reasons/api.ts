import { getJson, patchJson, postJson, sendDelete } from "../../../api/client";
import type { CreatePriorityReasonResponse } from "../../../api/generated/CreatePriorityReasonResponse";
import type { PriorityReasonListResponse } from "../../../api/generated/PriorityReasonListResponse";
import type { PriorityReasonRequest } from "../../../api/generated/PriorityReasonRequest";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";

/** 우선 배정 사유를 모두 받는다. 이름 순서. 모든 역할이 볼 수 있다. */
export function listPriorityReasons(): Promise<PriorityReasonListResponse> {
  return getJson<PriorityReasonListResponse>("/api/priority-reasons");
}

/** 새 사유를 저장하고 새 id를 받는다. 관리자만. */
export function createPriorityReason(
  request: PriorityReasonRequest,
): Promise<CreatePriorityReasonResponse> {
  return postJson<CreatePriorityReasonResponse>(
    "/api/priority-reasons",
    request,
  );
}

/** 사유의 이름과 방식을 바꾸고 고친 뒤 모습을 받는다. 관리자만. */
export function updatePriorityReason(
  id: number,
  request: PriorityReasonRequest,
): Promise<PriorityReasonResponse> {
  return patchJson<PriorityReasonResponse>(
    `/api/priority-reasons/${id}`,
    request,
  );
}

/** 사유를 지운다. 신청에 붙인 적이 있으면 거부된다(BR-S10). 관리자만. */
export function deletePriorityReason(id: number): Promise<void> {
  return sendDelete(`/api/priority-reasons/${id}`);
}
