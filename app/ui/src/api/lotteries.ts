import { getJson, postJson, postNoContent } from "./client";
import type { CourseLotteryResponse } from "./generated/CourseLotteryResponse";
import type { SelectionListResponse } from "./generated/SelectionListResponse";

/** 이 학기 강좌마다 선발 단계(추첨 전, 추첨함, 마감)를 받는다. 강좌 id 순. */
export function listSelections(termId: number): Promise<SelectionListResponse> {
  return getJson<SelectionListResponse>(`/api/terms/${termId}/selections`);
}

/** 강좌 하나의 추첨 화면(신청자, 결과, 이 PC에서 추첨할 수 있는지)을 받는다. */
export function getLottery(courseId: number): Promise<CourseLotteryResponse> {
  return getJson<CourseLotteryResponse>(`/api/courses/${courseId}/lottery`);
}

/**
 * 강좌를 추첨한다. 이미 추첨했으면 다시 추첨한다(BR-L4).
 *
 * 결과는 호스트가 이 요청을 받은 순간 정한다(ADR 0016). 호스트 PC에서만 된다(BR-L6).
 */
export function runLottery(courseId: number): Promise<CourseLotteryResponse> {
  return postJson<CourseLotteryResponse>(
    `/api/courses/${courseId}/lottery`,
    undefined,
  );
}

/** 강좌의 선발을 마감한다(BR-L5). 마감 뒤에는 다시 추첨할 수 없다. */
export function closeSelection(courseId: number): Promise<void> {
  return postNoContent(`/api/courses/${courseId}/selection-closure`);
}
