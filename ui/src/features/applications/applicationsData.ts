import type { AdditionalOutcomeResponse } from "../../api/generated/AdditionalOutcomeResponse";
import type { ApplicationCountResponse } from "../../api/generated/ApplicationCountResponse";
import type { ClosedCourseResponse } from "../../api/generated/ClosedCourseResponse";
import type { ApplicationRuleCode } from "../../api/generated/ApplicationRuleCode";
import type { CoursePrioritiesRequest } from "../../api/generated/CoursePrioritiesRequest";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { RuleViolationResponse } from "../../api/generated/RuleViolationResponse";
import type { SubmittedResponse } from "../../api/generated/SubmittedResponse";
import type { TermResponse } from "../../api/generated/TermResponse";
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

/** 강좌 id로 찾는, 선발 마감한 강좌에 지금 추가 신청을 받으면 어떻게 되는지(BR-X2). */
export function additionalByCourse(
  closed: readonly ClosedCourseResponse[],
): ReadonlyMap<number, AdditionalOutcomeResponse> {
  return new Map(
    closed.map((course) => [course.courseId, course.nextAdditional]),
  );
}

/** 추가 신청 결과의 화면 글자. 예: `바로 당첨`, `대기 3번째` */
export function additionalLabel(outcome: AdditionalOutcomeResponse): string {
  return outcome.kind === "selected"
    ? "바로 당첨"
    : `대기 ${outcome.waitlistNumber}번째`;
}

/** 학기 종료일이 지나 추가 신청을 받지 않는지(BR-X1). `today`는 `YYYY-MM-DD`. */
export function isTermEnded(term: TermResponse, today: string): boolean {
  return today > term.endsOn;
}

/**
 * 신청을 저장한 뒤 보여 줄 알림 줄들.
 *
 * 보통 신청은 건수만, 추가 신청은 강좌마다 결과를 쓴다. 바로 당첨으로 미리 보았는데
 * 그사이 다른 PC가 자리를 채워 대기로 받았으면 그렇게 알린다.
 *
 * @param previews 신청할 때 화면에 보였던 추가 신청 미리 보기. 강좌 id로 찾는다.
 */
export function submittedNotice(
  items: readonly SubmittedResponse[],
  previews: ReadonlyMap<number, AdditionalOutcomeResponse>,
  courses: readonly CourseResponse[],
): string[] {
  const nameOf = (id: number) => {
    const course = courses.find((candidate) => candidate.id === id);
    return course ? courseName(course) : "지워진 강좌";
  };
  const regular = items.filter((item) => item.additional === null).length;
  const lines = regular > 0 ? [`${regular}건 신청했습니다`] : [];
  for (const item of items) {
    const outcome = item.additional;
    if (outcome === null) {
      continue;
    }
    const name = nameOf(item.courseId);
    const preview = previews.get(item.courseId);
    if (preview?.kind === "selected" && outcome.kind === "waitlisted") {
      lines.push(
        `${name}: 자리가 방금 찼습니다. ${additionalLabel(outcome)}로 받았습니다`,
      );
    } else {
      lines.push(`${name}: 추가 신청, ${additionalLabel(outcome)}`);
    }
  }
  return lines;
}
