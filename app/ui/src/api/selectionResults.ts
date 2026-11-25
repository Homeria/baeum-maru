import { getJson, postNoContent } from "./client";
import type { ClosedCourseListResponse } from "./generated/ClosedCourseListResponse";
import type { ContactResultCode } from "./generated/ContactResultCode";
import type { CourseResultResponse } from "./generated/CourseResultResponse";

/** 이 학기에서 선발을 마감한 강좌들의 인원과 빈자리를 받는다. 강좌 id 순. */
export function listClosedCourses(
  termId: number,
): Promise<ClosedCourseListResponse> {
  return getJson<ClosedCourseListResponse>(
    `/api/terms/${termId}/selection-results`,
  );
}

/**
 * 마감한 강좌 하나의 선발 결과(당첨, 대기, 포기)를 받는다.
 *
 * 대기자 연락처는 포기·연락을 남길 수 있는 역할에게만 온다(BR-W2).
 */
export function getCourseResult(
  courseId: number,
): Promise<CourseResultResponse> {
  return getJson<CourseResultResponse>(
    `/api/courses/${courseId}/selection-result`,
  );
}

/** 당첨이나 대기인 신청을 포기로 바꾼다(BR-C3). 사유는 선택. */
export function withdraw(
  applicationId: number,
  reason: string | null,
): Promise<void> {
  return postNoContent(`/api/applications/${applicationId}/withdrawal`, {
    reason,
  });
}

/**
 * 대기자에게 연락한 결과를 남긴다(BR-W2~W5).
 *
 * 수락은 빈자리가 있을 때만 당첨이 된다. 없으면 `NO_OPEN_SEAT` 오류.
 */
export function recordContact(
  applicationId: number,
  result: ContactResultCode,
): Promise<void> {
  return postNoContent(`/api/applications/${applicationId}/contacts`, {
    result,
  });
}

/** 잘못 처리한 포기를 포기 전 상태로 되돌린다(BR-W8). 관리자만, 사유 필수. */
export function correctWithdrawal(
  applicationId: number,
  reason: string,
): Promise<void> {
  return postNoContent(
    `/api/applications/${applicationId}/withdrawal-correction`,
    { reason },
  );
}
