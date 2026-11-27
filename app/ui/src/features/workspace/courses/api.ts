import { getJson, patchJson, postJson, sendDelete } from "../../../api/client";
import type { ApplicationCountListResponse } from "../../../api/generated/ApplicationCountListResponse";
import type { CourseListResponse } from "../../../api/generated/CourseListResponse";
import type { CourseRequest } from "../../../api/generated/CourseRequest";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { CreateCourseResponse } from "../../../api/generated/CreateCourseResponse";
import type { SelectionListResponse } from "../../../api/generated/SelectionListResponse";

/** 학기의 강좌를 모두 받는다. 과목 이름, 등록 순서. */
export function listCourses(termId: number): Promise<CourseListResponse> {
  return getJson<CourseListResponse>(`/api/terms/${termId}/courses`);
}

/** 이 학기 강좌별 신청 인원을 받는다. 신청이 없는 강좌는 빠진다. */
export function listApplicationCounts(
  termId: number,
): Promise<ApplicationCountListResponse> {
  return getJson<ApplicationCountListResponse>(
    `/api/terms/${termId}/application-counts`,
  );
}

/** 이 학기 강좌마다 선발 단계(추첨 전, 추첨함, 마감)를 받는다. 강좌 id 순. */
export function listSelections(termId: number): Promise<SelectionListResponse> {
  return getJson<SelectionListResponse>(`/api/terms/${termId}/selections`);
}

/** 학기에 새 강좌를 저장하고 새 id를 받는다. */
export function createCourse(
  termId: number,
  request: CourseRequest,
): Promise<CreateCourseResponse> {
  return postJson<CreateCourseResponse>(
    `/api/terms/${termId}/courses`,
    request,
  );
}

/** 강좌의 칸과 수업 시간을 모두 바꾸고 고친 뒤 모습을 받는다. */
export function updateCourse(
  id: number,
  request: CourseRequest,
): Promise<CourseResponse> {
  return patchJson<CourseResponse>(`/api/courses/${id}`, request);
}

/** 강좌와 그 수업 시간을 지운다. */
export function deleteCourse(id: number): Promise<void> {
  return sendDelete(`/api/courses/${id}`);
}
