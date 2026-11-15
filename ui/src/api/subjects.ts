import { getJson, patchJson, postJson, sendDelete } from "./client";
import type { CreateSubjectResponse } from "./generated/CreateSubjectResponse";
import type { SubjectListResponse } from "./generated/SubjectListResponse";
import type { SubjectRequest } from "./generated/SubjectRequest";
import type { SubjectResponse } from "./generated/SubjectResponse";

/** 과목을 모두 받는다. 분류, 이름 순서. */
export function listSubjects(): Promise<SubjectListResponse> {
  return getJson<SubjectListResponse>("/api/subjects");
}

/** 새 과목을 저장하고 새 id를 받는다. */
export function createSubject(
  request: SubjectRequest,
): Promise<CreateSubjectResponse> {
  return postJson<CreateSubjectResponse>("/api/subjects", request);
}

/** 과목의 칸을 모두 바꾸고 고친 뒤 모습을 받는다. */
export function updateSubject(
  id: number,
  request: SubjectRequest,
): Promise<SubjectResponse> {
  return patchJson<SubjectResponse>(`/api/subjects/${id}`, request);
}

/** 과목을 지운다. */
export function deleteSubject(id: number): Promise<void> {
  return sendDelete(`/api/subjects/${id}`);
}
