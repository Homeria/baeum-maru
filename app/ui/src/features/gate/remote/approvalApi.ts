import { getJson, postJson } from "../../../api/client";
import type { ApprovalRequestResponse } from "../../../api/generated/ApprovalRequestResponse";
import type { ApprovalStatusResponse } from "../../../api/generated/ApprovalStatusResponse";

/** BR-K15: 이 PC의 승인을 호스트에 요청한다. 기기 증표와 승인 번호를 받는다. */
export function requestApproval(
  name: string,
): Promise<ApprovalRequestResponse> {
  return postJson<ApprovalRequestResponse>("/api/approval/request", { name });
}

/** 요청에 실린 기기 증표의 승인 요청 상태. */
export function approvalStatus(): Promise<ApprovalStatusResponse> {
  return getJson<ApprovalStatusResponse>("/api/approval/status");
}
