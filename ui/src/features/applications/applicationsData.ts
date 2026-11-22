import type { ApplicationCountResponse } from "../../api/generated/ApplicationCountResponse";
import type { ApplicationRuleCode } from "../../api/generated/ApplicationRuleCode";
import type { CoursePrioritiesRequest } from "../../api/generated/CoursePrioritiesRequest";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { RuleViolationResponse } from "../../api/generated/RuleViolationResponse";
import { courseName } from "../courses/courseFormData";

/** 신청 규칙의 화면 이름. 용어집의 "신청 규칙 종류"를 따른다. */
export const RULE_LABELS: Record<ApplicationRuleCode, string> = {
  no_duplicate: "같은 강좌 두 번",
  no_time_overlap: "시간 겹침",
  no_same_subject: "같은 과목 중복",
};

/** 강좌 id로 신청 인원을 찾는 표. 목록에 없는 강좌는 0명이다. */
export function countsByCourse(
  counts: readonly ApplicationCountResponse[],
): ReadonlyMap<number, number> {
  return new Map(counts.map((count) => [count.courseId, count.count]));
}

/** 신청 인원과 정원. 예: `12/20` */
export function countLabel(count: number, capacity: number): string {
  return `${count}/${capacity}`;
}

/** 고른 강좌 목록에서 `id`를 넣거나 뺀다. 고른 순서를 지킨다. */
export function toggleChoice(
  chosen: readonly number[],
  id: number,
): readonly number[] {
  return chosen.includes(id)
    ? chosen.filter((chosenId) => chosenId !== id)
    : [...chosen, id];
}

/**
 * 어긴 규칙 한 줄. 예: `한글교실: 시간 겹침 (요가 초급)`
 *
 * 같은 강좌 두 번(BR-A4)은 상대가 자기 자신이라 `요가 초급: 이미 신청한 강좌`로 쓴다.
 */
export function violationMessage(
  violation: RuleViolationResponse,
  courses: readonly CourseResponse[],
): string {
  const nameOf = (id: number) => {
    const course = courses.find((candidate) => candidate.id === id);
    return course ? courseName(course) : "지워진 강좌";
  };
  const name = nameOf(violation.courseId);
  if (violation.rule === "no_duplicate") {
    return `${name}: 이미 신청한 강좌`;
  }
  return `${name}: ${RULE_LABELS[violation.rule]} (${nameOf(violation.otherCourseId)})`;
}

/** 강좌 id로 찾는, 고른 강좌에 붙일 우선 배정 사유 id들. */
export type ChosenPriorities = ReadonlyMap<number, readonly number[]>;

/** id 목록에서 `id`를 넣거나 뺀다. 늘 작은 id부터. */
export function toggleId(
  ids: readonly number[],
  id: number,
): readonly number[] {
  return ids.includes(id)
    ? ids.filter((other) => other !== id)
    : [...ids, id].sort((a, b) => a - b);
}

/** 고른 강좌 `courseId`의 사유 `reasonId`를 넣거나 뺀다. 사유가 다 빠진 강좌는 지운다. */
export function togglePriority(
  priorities: ChosenPriorities,
  courseId: number,
  reasonId: number,
): ChosenPriorities {
  const next = new Map(priorities);
  const ids = toggleId(priorities.get(courseId) ?? [], reasonId);
  if (ids.length === 0) {
    next.delete(courseId);
  } else {
    next.set(courseId, ids);
  }
  return next;
}

/** 신청 요청의 `priorities`. 고른 강좌의 것만, 고른 순서로 싣는다(BR-S12). */
export function priorityRequests(
  chosen: readonly number[],
  priorities: ChosenPriorities,
): CoursePrioritiesRequest[] {
  return chosen.flatMap((courseId) => {
    const reasonIds = priorities.get(courseId) ?? [];
    return reasonIds.length === 0
      ? []
      : [{ courseId, reasonIds: [...reasonIds] }];
  });
}
