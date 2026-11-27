import { getJson, patchJson, postJson, sendDelete } from "../../../api/client";
import type { CreateTermResponse } from "../../../api/generated/CreateTermResponse";
import type { TermListResponse } from "../../../api/generated/TermListResponse";
import type { TermRequest } from "../../../api/generated/TermRequest";
import type { TermResponse } from "../../../api/generated/TermResponse";

/** 학기를 모두 받는다. 시작일이 늦은 학기부터. */
export function listTerms(): Promise<TermListResponse> {
  return getJson<TermListResponse>("/api/terms");
}

/** 새 학기를 저장하고 새 id를 받는다. */
export function createTerm(request: TermRequest): Promise<CreateTermResponse> {
  return postJson<CreateTermResponse>("/api/terms", request);
}

/** 학기의 칸을 모두 바꾸고 고친 뒤 모습을 받는다. */
export function updateTerm(
  id: number,
  request: TermRequest,
): Promise<TermResponse> {
  return patchJson<TermResponse>(`/api/terms/${id}`, request);
}

/** 학기를 지운다. 강좌가 있으면 거절된다 (BR-T5). */
export function deleteTerm(id: number): Promise<void> {
  return sendDelete(`/api/terms/${id}`);
}
