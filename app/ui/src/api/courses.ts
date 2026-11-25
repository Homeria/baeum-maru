import { getJson, patchJson, postJson, sendDelete } from "./client";
import type { CourseListResponse } from "./generated/CourseListResponse";
import type { CourseRequest } from "./generated/CourseRequest";
import type { CourseResponse } from "./generated/CourseResponse";
import type { CreateCourseResponse } from "./generated/CreateCourseResponse";

/** 학기의 강좌를 모두 받는다. 과목 이름, 등록 순서. */
export function listCourses(termId: number): Promise<CourseListResponse> {
  return getJson<CourseListResponse>(`/api/terms/${termId}/courses`);
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
