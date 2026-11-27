import { getJson, postJson, putJson, sendDelete } from "../../../api/client";
import type { ApplicationCountListResponse } from "../../../api/generated/ApplicationCountListResponse";
import type { ApplicationListResponse } from "../../../api/generated/ApplicationListResponse";
import type { DeleteApplicationRequest } from "../../../api/generated/DeleteApplicationRequest";
import type { SetPriorityReasonsResponse } from "../../../api/generated/SetPriorityReasonsResponse";
import type { SubmitApplicationsRequest } from "../../../api/generated/SubmitApplicationsRequest";
import type { SubmitApplicationsResponse } from "../../../api/generated/SubmitApplicationsResponse";

/** 회원이 이 학기에 낸 유효한 신청을 받는다. 접수한 순서. */
export function listApplications(
  termId: number,
  memberId: number,
): Promise<ApplicationListResponse> {
  return getJson<ApplicationListResponse>(
    `/api/terms/${termId}/applications?memberId=${memberId}`,
  );
}

/** 이 학기 강좌별 신청 인원을 받는다. 신청이 없는 강좌는 빠진다. */
export function listApplicationCounts(
  termId: number,
): Promise<ApplicationCountListResponse> {
  return getJson<ApplicationCountListResponse>(
    `/api/terms/${termId}/application-counts`,
  );
}

/**
 * 회원 한 명의 신청 여러 건을 한 번에 저장한다.
 *
 * 규칙을 어기면 `RULES_BROKEN` 오류의 `violations`에 어긴 규칙이 모두 온다(BR-A5).
 */
export function submitApplications(
  termId: number,
  request: SubmitApplicationsRequest,
): Promise<SubmitApplicationsResponse> {
  return postJson<SubmitApplicationsResponse>(
    `/api/terms/${termId}/applications`,
    request,
  );
}

/** 접수됨 신청을 지운다. 호스트가 삭제 기록을 남긴다(BR-D3). */
export function deleteApplication(
  id: number,
  request: DeleteApplicationRequest,
): Promise<void> {
  return sendDelete(`/api/applications/${id}`, request);
}

/**
 * 신청의 우선 배정 사유를 `reasonIds`로 바꾼다. 빈 목록이면 모두 뗀다(BR-S12).
 *
 * 추첨한 강좌면 그 추첨이 취소되고 `lotteryCancelled`가 `true`다(BR-S13).
 */
export function setPriorityReasons(
  id: number,
  reasonIds: number[],
): Promise<SetPriorityReasonsResponse> {
  return putJson<SetPriorityReasonsResponse>(
    `/api/applications/${id}/priority-reasons`,
    { reasonIds },
  );
}
